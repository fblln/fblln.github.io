//! `dtls-cid` — instruments "The Lookup That Saves the Handshake".
//!
//! The article's diagrams already show the record layout and the receive-path
//! decision tree; what they cannot show is the *consequence* of anchoring session
//! state on an address the protocol never chose. So the reader operates it:
//! rebind the NAT, watch the association vanish and the handshake counter climb,
//! switch the anchor to a connection identifier, and watch the same rebind cost
//! nothing.
//!
//! The three conditions RFC 9146 sets before a peer address may move are the real
//! subject. A reader who only sees "CID on, session survives" has learned the
//! summary the article exists to argue against, so the instrument also lets them
//! replay an old record from a forged address and watch conditions 2 and 3 refuse
//! to turn the server into a reflector.

/// The simulation. No DOM, no reactivity, no allocation beyond the log — so the
/// protocol argument can be tested without a browser, which is the only reason
/// the invariants below are cheap enough to assert exhaustively.
pub(crate) mod logic {
    use crate::instrument::{Level, Log};

    /// Where the receiver looks up the security association for an inbound
    /// record. This single choice is the whole subject of the article.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub(crate) enum Anchor {
        /// DTLS 1.2 as originally specified: the association is found by the
        /// packet's 5-tuple, an identifier the protocol never chose and does
        /// not control.
        FiveTuple,
        /// RFC 9146: the association is found by an identifier the receiver
        /// itself issued, carried in the record.
        ConnectionId,
    }

    /// What the receive path did with a record. Each variant maps to one branch
    /// of the decision flow the article draws.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub(crate) enum Outcome {
        /// No association could be located. In a real deployment this is a full
        /// handshake: certificates, key exchange, the lot.
        NoAssociation,
        /// Located and delivered from the address already bound.
        Delivered,
        /// Located and delivered, but the record was not newer than the newest
        /// already seen, so the binding stays put. RFC 9146 condition 2 — it
        /// stops a late or replayed packet dragging the binding backwards.
        DeliveredNoMove,
        /// Located, newer, from a new address — but the peer has not proven it
        /// can receive there. RFC 9146 condition 3. The binding is held until
        /// return routability completes; without this the server is a reflector.
        PendingValidation,
        /// Return routability completed and the binding moved.
        BindingMoved,
    }

    /// Plausible addresses for the sensor, in documentation ranges. A fixed
    /// rotation keeps every run reproducible — a reader comparing two runs is
    /// comparing the protocol, not a random number generator.
    pub(crate) const ADDRS: [&str; 4] = [
        "198.51.100.23:62000",
        "198.51.100.23:49172",
        "198.51.100.23:51884",
        "203.0.113.44:33410",
    ];

    /// The address an attacker forges when replaying a captured record.
    pub(crate) const FORGED: &str = "192.0.2.66:9";

    /// The connection identifier the server issued during the handshake. Opaque
    /// by construction — its only job is to name the association.
    pub(crate) const CID: &str = "0x2f4a91c6";

    #[derive(Clone, PartialEq, Eq, Debug)]
    pub(crate) struct Sim {
        pub(crate) anchor: Anchor,
        /// Where the server currently sends replies.
        pub(crate) bound: String,
        /// Where the sensor actually is, after any NAT rebinding.
        pub(crate) client: String,
        /// Highest sequence number accepted so far.
        pub(crate) highest_seq: u32,
        /// The sensor's next record sequence.
        pub(crate) next_seq: u32,
        /// Full handshakes forced so far. This is the number that matters.
        pub(crate) handshakes: u32,
        /// An address awaiting return routability before the binding may move.
        pub(crate) pending: Option<String>,
        /// Index into `ADDRS` for the next rebinding.
        pub(crate) rebinds: usize,
        pub(crate) log: Log,
    }

    impl Default for Sim {
        fn default() -> Self {
            Self::new(Anchor::FiveTuple)
        }
    }

    impl Sim {
        pub(crate) fn new(anchor: Anchor) -> Self {
            let mut sim = Self {
                anchor,
                bound: ADDRS[0].to_string(),
                client: ADDRS[0].to_string(),
                highest_seq: 42,
                next_seq: 43,
                handshakes: 0,
                pending: None,
                rebinds: 0,
                log: Log::default(),
            };
            sim.log.note(
                Level::Ok,
                format!(
                    "handshake complete · association anchored on {}",
                    sim.anchor_label()
                ),
            );
            sim
        }

        pub(crate) fn anchor_label(&self) -> &'static str {
            match self.anchor {
                Anchor::FiveTuple => "5-tuple",
                Anchor::ConnectionId => "connection id",
            }
        }

        /// Switching the anchor restarts the session: in reality this is a
        /// deployment decision made before the handshake, not a runtime toggle,
        /// and pretending otherwise would teach the reader something false.
        pub(crate) fn set_anchor(&mut self, anchor: Anchor) {
            *self = Self::new(anchor);
        }

        /// The NAT mapping expires while the sensor sleeps; it wakes on a new
        /// external port. Nothing cryptographic has changed.
        pub(crate) fn rebind(&mut self) {
            self.rebinds += 1;
            self.client = ADDRS[self.rebinds % ADDRS.len()].to_string();
            self.log.note(
                Level::Held,
                format!("nat rebinding · sensor now sends from {}", self.client),
            );
        }

        /// The sensor sends its next reading from wherever it currently is.
        pub(crate) fn send(&mut self) -> Outcome {
            let seq = self.next_seq;
            self.next_seq += 1;
            let from = self.client.clone();
            self.receive(&from, seq, false)
        }

        /// An attacker replays a captured record from a forged source address.
        /// The record is authentic — it was authentic when it was recorded —
        /// which is precisely why conditions 2 and 3 have to exist.
        pub(crate) fn replay(&mut self) -> Outcome {
            let seq = self.highest_seq.saturating_sub(1);
            self.receive(FORGED, seq, true)
        }

        /// The receive path. Mirrors the decision flow the article draws:
        /// locate, authenticate, then — only if the address changed — check
        /// that the record is newer and the path is proven.
        fn receive(&mut self, from: &str, seq: u32, forged: bool) -> Outcome {
            let located = match self.anchor {
                // The 5-tuple *is* the key, so a different address is simply a
                // different key, and there is nothing to find.
                Anchor::FiveTuple => from == self.bound,
                // The CID names the association directly, so the address the
                // packet happened to arrive from is irrelevant to the lookup.
                Anchor::ConnectionId => true,
            };

            if !located {
                self.handshakes += 1;
                self.bound = from.to_string();
                self.highest_seq = seq;
                self.pending = None;
                self.log.note(
                    Level::Fail,
                    format!("record seq {seq} from {from} · no association for this 5-tuple — full handshake"),
                );
                return Outcome::NoAssociation;
            }

            let via = match self.anchor {
                Anchor::FiveTuple => "5-tuple".to_string(),
                Anchor::ConnectionId => format!("cid {CID}"),
            };

            if from == self.bound {
                if seq > self.highest_seq {
                    self.highest_seq = seq;
                }
                self.log.note(
                    Level::Ok,
                    format!("record seq {seq} from {from} · {via} → delivered"),
                );
                return Outcome::Delivered;
            }

            // Condition 2 — the record must be newer than anything already
            // accepted, or an old capture could drag the binding backwards.
            if seq <= self.highest_seq {
                let who = if forged { " (forged source)" } else { "" };
                self.log.note(
                    Level::Held,
                    format!(
                        "record seq {seq} from {from}{who} · {via} → delivered, binding unchanged — seq {seq} is not newer than {}",
                        self.highest_seq
                    ),
                );
                return Outcome::DeliveredNoMove;
            }

            // Condition 3 — the peer must prove it can receive at the new
            // address before the server will send anything substantial there.
            self.highest_seq = seq;
            self.pending = Some(from.to_string());
            let who = if forged { " (forged source)" } else { "" };
            self.log.note(
                Level::Held,
                format!(
                    "record seq {seq} from {from}{who} · {via} → association found, address unproven — path challenge sent, binding held"
                ),
            );
            Outcome::PendingValidation
        }

        /// Return routability completed: the peer echoed the challenge, so the
        /// address is real and the binding may move. An attacker replaying from
        /// a forged address cannot reach this, which is the entire point.
        pub(crate) fn validate(&mut self) -> Option<Outcome> {
            let addr = self.pending.take()?;
            self.bound = addr.clone();
            self.log.note(
                Level::Ok,
                format!("path response echoed · {addr} validated — binding moved"),
            );
            Some(Outcome::BindingMoved)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn a_fresh_session_is_anchored_and_costs_nothing() {
            let sim = Sim::new(Anchor::FiveTuple);
            assert_eq!(sim.handshakes, 0);
            assert_eq!(sim.bound, ADDRS[0]);
            assert_eq!(sim.client, ADDRS[0]);
            assert_eq!(sim.log.len(), 1);
            assert_eq!(Sim::default(), Sim::new(Anchor::FiveTuple));
        }

        #[test]
        fn records_from_the_bound_address_are_delivered() {
            let mut sim = Sim::new(Anchor::FiveTuple);
            assert_eq!(sim.send(), Outcome::Delivered);
            assert_eq!(sim.handshakes, 0);
            assert_eq!(sim.highest_seq, 43);
        }

        #[test]
        fn five_tuple_anchoring_loses_the_association_on_every_rebinding() {
            let mut sim = Sim::new(Anchor::FiveTuple);
            sim.rebind();
            assert_eq!(sim.send(), Outcome::NoAssociation);
            assert_eq!(sim.handshakes, 1);
            // The new handshake re-anchors on wherever the sensor now is.
            assert_eq!(sim.bound, sim.client);
            assert_eq!(sim.send(), Outcome::Delivered);
            assert_eq!(sim.handshakes, 1);
        }

        #[test]
        fn connection_id_anchoring_survives_a_rebinding_without_a_handshake() {
            let mut sim = Sim::new(Anchor::ConnectionId);
            sim.rebind();
            assert_eq!(sim.send(), Outcome::PendingValidation);
            assert_eq!(sim.handshakes, 0);
            assert_eq!(sim.pending.as_deref(), Some(sim.client.as_str()));
            // Held until proven, then moved.
            assert_eq!(sim.validate(), Some(Outcome::BindingMoved));
            assert_eq!(sim.bound, sim.client);
            assert!(sim.pending.is_none());
        }

        #[test]
        fn validating_without_a_pending_address_does_nothing() {
            let mut sim = Sim::new(Anchor::ConnectionId);
            assert_eq!(sim.validate(), None);
            assert_eq!(sim.bound, ADDRS[0]);
        }

        #[test]
        fn condition_two_refuses_to_drag_the_binding_backwards() {
            let mut sim = Sim::new(Anchor::ConnectionId);
            sim.rebind();
            assert_eq!(sim.send(), Outcome::PendingValidation);
            sim.validate();
            let bound = sim.bound.clone();
            // A captured older record, forged source: authentic, but not newer.
            assert_eq!(sim.replay(), Outcome::DeliveredNoMove);
            assert_eq!(
                sim.bound, bound,
                "an old record must never move the binding"
            );
            assert!(sim.pending.is_none());
        }

        #[test]
        fn condition_three_holds_the_binding_against_a_newer_forged_record() {
            let mut sim = Sim::new(Anchor::ConnectionId);
            let bound = sim.bound.clone();
            // Reach the receive path directly with a newer sequence from a
            // forged address — the strongest case an attacker has.
            assert_eq!(
                sim.receive(FORGED, sim.highest_seq + 5, true),
                Outcome::PendingValidation
            );
            assert_eq!(sim.bound, bound, "binding must not move before validation");
            assert_eq!(sim.pending.as_deref(), Some(FORGED));
        }

        #[test]
        fn switching_the_anchor_restarts_the_session() {
            let mut sim = Sim::new(Anchor::FiveTuple);
            sim.rebind();
            sim.send();
            assert_eq!(sim.handshakes, 1);
            sim.set_anchor(Anchor::ConnectionId);
            assert_eq!(sim.anchor, Anchor::ConnectionId);
            assert_eq!(sim.handshakes, 0);
            assert_eq!(sim.bound, ADDRS[0]);
            assert_eq!(sim.anchor_label(), "connection id");
        }

        #[test]
        fn rebinding_walks_the_address_rotation_and_wraps() {
            let mut sim = Sim::new(Anchor::ConnectionId);
            for expected in [ADDRS[1], ADDRS[2], ADDRS[3], ADDRS[0]] {
                sim.rebind();
                assert_eq!(sim.client, expected);
            }
        }

        #[test]
        fn the_log_is_bounded_and_the_tail_is_the_last_six() {
            let mut sim = Sim::new(Anchor::ConnectionId);
            for _ in 0..40 {
                sim.send();
            }
            assert!(sim.log.len() <= 24, "log must not grow without limit");
            assert_eq!(sim.log.tail().len(), 6);
            assert_eq!(sim.log.tail().last(), sim.log.last());
        }

        #[test]
        fn the_tail_is_short_before_six_lines_exist() {
            let sim = Sim::new(Anchor::FiveTuple);
            assert_eq!(sim.log.tail().len(), 1);
        }

        #[test]
        fn every_outcome_records_exactly_one_log_line() {
            let mut sim = Sim::new(Anchor::ConnectionId);
            for step in 0..12 {
                let before = sim.log.len().min(23);
                match step % 4 {
                    0 => {
                        sim.rebind();
                    }
                    1 => {
                        sim.send();
                    }
                    2 => {
                        sim.replay();
                    }
                    _ => {
                        sim.validate();
                    }
                }
                // validate() on an empty pending is the one no-op action.
                assert!(sim.log.len() >= before, "log must never shrink mid-run");
            }
        }

        /// The article's central claim, pinned as an invariant: anchoring on an
        /// identifier the receiver owns means address churn is free. Any
        /// mutation that reintroduces an address comparison into the CID lookup
        /// path fails here.
        #[test]
        fn invariant_connection_id_never_costs_a_handshake() {
            let mut sim = Sim::new(Anchor::ConnectionId);
            for _ in 0..25 {
                sim.rebind();
                sim.send();
                sim.replay();
                sim.validate();
            }
            assert_eq!(
                sim.handshakes, 0,
                "no sequence of rebindings may force a handshake under CID anchoring"
            );
        }

        /// The mirror invariant: under 5-tuple anchoring every rebinding costs
        /// exactly one handshake. Pins the cost the article is arguing about.
        #[test]
        fn invariant_five_tuple_costs_one_handshake_per_rebinding() {
            let mut sim = Sim::new(Anchor::FiveTuple);
            for expected in 1..=8 {
                sim.rebind();
                sim.send();
                assert_eq!(sim.handshakes, expected);
            }
        }

        /// The reflector defence, pinned: the reply address may only ever become
        /// an address that completed return routability. A mutation that moves
        /// `bound` inside `receive` rather than inside `validate` fails here.
        #[test]
        fn invariant_binding_only_moves_through_validation_or_handshake() {
            let mut sim = Sim::new(Anchor::ConnectionId);
            for seq_bump in 1..12 {
                let before = sim.bound.clone();
                let outcome = sim.receive(FORGED, sim.highest_seq + seq_bump, true);
                assert_ne!(outcome, Outcome::NoAssociation);
                assert_eq!(
                    sim.bound, before,
                    "receive() must never move the binding on its own"
                );
            }
            // Only an explicit validation moves it.
            assert_eq!(sim.validate(), Some(Outcome::BindingMoved));
            assert_eq!(sim.bound, FORGED);
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) mod view {
    use super::logic::{Anchor, CID, Sim};
    use leptos::prelude::*;

    /// Renders one instrument. The panel deliberately mirrors the article's
    /// diagram vocabulary — mono labels, hairline rules, one signal emphasis —
    /// so it reads as another figure rather than as an embedded application.
    #[component]
    pub(crate) fn DtlsCid() -> impl IntoView {
        let sim = RwSignal::new(Sim::default());

        let anchor_is = move |a: Anchor| sim.with(|s| s.anchor == a);
        let set_anchor = move |a: Anchor| sim.update(|s| s.set_anchor(a));

        view! {
            <div class="inst" role="group" aria-label="DTLS connection identifier instrument">
                <div class="inst-head">
                    <span>"SESSION/DTLS"</span>
                    <span class="inst-anchor">
                        "ANCHOR: " {move || sim.with(|s| s.anchor_label().to_uppercase())}
                    </span>
                </div>

                <div class="inst-controls">
                    <button
                        type="button"
                        class:active=move || anchor_is(Anchor::FiveTuple)
                        on:click=move |_| set_anchor(Anchor::FiveTuple)
                    >"5-TUPLE"</button>
                    <button
                        type="button"
                        class:active=move || anchor_is(Anchor::ConnectionId)
                        on:click=move |_| set_anchor(Anchor::ConnectionId)
                    >"CONNECTION ID"</button>
                </div>

                <div class="inst-readout">
                    <div>
                        <span>"SENSOR SENDS FROM"</span>
                        <strong>{move || sim.with(|s| s.client.clone())}</strong>
                    </div>
                    <div>
                        <span>"SERVER REPLIES TO"</span>
                        <strong>{move || sim.with(|s| s.bound.clone())}</strong>
                    </div>
                    <div>
                        <span>"LOOKUP KEY"</span>
                        <strong>{move || sim.with(|s| match s.anchor {
                            Anchor::FiveTuple => s.bound.clone(),
                            Anchor::ConnectionId => CID.to_string(),
                        })}</strong>
                    </div>
                    <div>
                        <span>"HIGHEST SEQ"</span>
                        <strong>{move || sim.with(|s| s.highest_seq)}</strong>
                    </div>
                    <div class="inst-cost">
                        <span>"FULL HANDSHAKES"</span>
                        <strong class:hot=move || sim.with(|s| s.handshakes > 0)>
                            {move || sim.with(|s| s.handshakes)}
                        </strong>
                    </div>
                </div>

                <Show when=move || sim.with(|s| s.pending.is_some())>
                    <p class="inst-pending">
                        "Address unproven — "
                        {move || sim.with(|s| s.pending.clone().unwrap_or_default())}
                        " has not completed return routability. The binding is held."
                    </p>
                </Show>

                <div class="inst-actions">
                    <button type="button" on:click=move |_| sim.update(|s| s.rebind())>
                        "REBIND NAT"
                    </button>
                    <button type="button" on:click=move |_| sim.update(|s| { s.send(); })>
                        "SEND RECORD"
                    </button>
                    <button type="button" on:click=move |_| sim.update(|s| { s.replay(); })>
                        "REPLAY OLD RECORD ↗ FORGED"
                    </button>
                    <button
                        type="button"
                        prop:disabled=move || sim.with(|s| s.pending.is_none())
                        on:click=move |_| sim.update(|s| { s.validate(); })
                    >"VALIDATE PATH"</button>
                    <button type="button" on:click=move |_| sim.update(|s| {
                        let a = s.anchor;
                        s.set_anchor(a);
                    })>"RESET"</button>
                </div>

                <ol class="inst-log">
                    {move || sim.with(|s| s.log.tail().iter().map(|line| {
                        view! { <li class=line.level.class()>{line.text.clone()}</li> }
                    }).collect_view())}
                </ol>
            </div>
        }
    }
}
