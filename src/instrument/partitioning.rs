//! `partitioning` — key partitioning, and what a partition count change costs.
//!
//! Two ideas usually taught apart are the same mechanism with one part swapped:
//! a set of keys, a set of buckets, and a function assigning one to the other.
//! Kafka's default partitioner uses `murmur2(key) % partitions`; a consistent
//! hash ring places buckets on a 32-bit circle and walks clockwise. Everything
//! interesting is in what happens when the bucket set changes.
//!
//! Readers reliably get this wrong in one specific, expensive way: they add a
//! partition to a Kafka topic to increase throughput and are surprised that
//! per-key ordering breaks. It breaks because almost every key changes
//! partition — `h % 4` and `h % 5` agree for roughly a fifth of hashes — so
//! messages for a given key now live behind two partitions with no ordering
//! relationship between them. A static diagram can draw either scheme, but the
//! comparison is the lesson, and comparison is exactly what a diagram cannot do.
//!
//! So the instrument reports one number against its own floor: how many keys
//! moved, beside the `1/n` an ideal scheme would have to move anyway. Modulo
//! lands near 80% where the floor is 20%. The ring lands near the floor. The
//! virtual-node count is exposed too, because a naive ring with one point per
//! partition is genuinely lumpy, and that is the honest version of the story.

pub(crate) mod logic {
    use crate::instrument::{Level, Log};

    /// Keys are device identifiers because that is the shape of the problem in
    /// the systems this site is about — a fleet keyed by vehicle, partitioned
    /// so each vehicle's telemetry stays ordered.
    pub(crate) const KEY_COUNT: usize = 96;
    pub(crate) const MIN_PARTITIONS: usize = 2;
    pub(crate) const MAX_PARTITIONS: usize = 12;
    pub(crate) const START_PARTITIONS: usize = 4;

    /// Virtual nodes per partition on the ring. 1 exposes the skew a naive ring
    /// suffers; 128 is the order of magnitude real systems use.
    pub(crate) const VNODE_CHOICES: [usize; 3] = [1, 16, 128];
    pub(crate) const START_VNODES: usize = 128;

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub(crate) enum Strategy {
        /// `toPositive(murmur2(key)) % numPartitions` — Kafka's DefaultPartitioner.
        Modulo,
        /// Buckets placed on a 32-bit ring; a key takes the first bucket
        /// clockwise from its own hash.
        Ring,
    }

    impl Strategy {
        pub(crate) fn label(self) -> &'static str {
            match self {
                Strategy::Modulo => "kafka default · murmur2 % n",
                Strategy::Ring => "consistent hash ring",
            }
        }
    }

    /// Kafka's murmur2 seed (`org.apache.kafka.common.utils.Utils`).
    const MURMUR2_SEED: u32 = 0x9747_b28c;

    /// MurmurHash2, 32-bit, as Kafka implements it. Kafka works in signed
    /// arithmetic and then masks the sign bit off; the bit pattern is identical
    /// either way, so this stays in `u32` and masks at the call site.
    ///
    /// The exact algorithm matters for which partition a given key lands in.
    /// It does not matter for the lesson: every property this instrument
    /// reports holds for any well-distributed hash, which is worth knowing
    /// before trusting the specific numbers on screen.
    pub(crate) fn murmur2(data: &[u8]) -> u32 {
        const M: u32 = 0x5bd1_e995;
        const R: u32 = 24;

        let len = data.len();
        let mut h: u32 = MURMUR2_SEED ^ (len as u32);

        let body = len & !3;
        let mut i = 0;
        while i < body {
            let mut k = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
            k = k.wrapping_mul(M);
            k ^= k >> R;
            k = k.wrapping_mul(M);
            h = h.wrapping_mul(M);
            h ^= k;
            i += 4;
        }

        // Tail bytes, folded in high-to-low exactly as the reference does.
        match len & 3 {
            3 => {
                h ^= (data[i + 2] as u32) << 16;
                h ^= (data[i + 1] as u32) << 8;
                h ^= data[i] as u32;
                h = h.wrapping_mul(M);
            }
            2 => {
                h ^= (data[i + 1] as u32) << 8;
                h ^= data[i] as u32;
                h = h.wrapping_mul(M);
            }
            1 => {
                h ^= data[i] as u32;
                h = h.wrapping_mul(M);
            }
            _ => {}
        }

        h ^= h >> 13;
        h = h.wrapping_mul(M);
        h ^= h >> 15;
        h
    }

    /// Kafka's `toPositive`: clear the sign bit rather than take an absolute
    /// value, so `i32::MIN` does not overflow into itself.
    fn to_positive(h: u32) -> u32 {
        h & 0x7fff_ffff
    }

    pub(crate) fn key_name(index: usize) -> String {
        format!("vehicle-{index:04}")
    }

    #[derive(Clone, PartialEq, Eq, Debug)]
    pub(crate) struct Cluster {
        pub(crate) strategy: Strategy,
        pub(crate) partitions: usize,
        pub(crate) vnodes: usize,
        /// Key index → partition.
        pub(crate) assignment: Vec<usize>,
        /// Keys that changed partition on the most recent change.
        pub(crate) relocated: Vec<usize>,
        /// Partition count before the most recent change, for the ideal floor.
        pub(crate) previous_partitions: usize,
        pub(crate) log: Log,
        /// Ring points, cached: rebuilding per key would be O(n log n) per
        /// lookup for no reason.
        points: Vec<(u32, usize)>,
        key_hashes: Vec<u32>,
    }

    impl Default for Cluster {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Cluster {
        pub(crate) fn new() -> Self {
            let key_hashes = (0..KEY_COUNT)
                .map(|i| murmur2(key_name(i).as_bytes()))
                .collect();
            let mut cluster = Self {
                strategy: Strategy::Modulo,
                partitions: START_PARTITIONS,
                vnodes: START_VNODES,
                assignment: Vec::new(),
                relocated: Vec::new(),
                previous_partitions: START_PARTITIONS,
                log: Log::default(),
                points: Vec::new(),
                key_hashes,
            };
            cluster.rebuild_points();
            cluster.assignment = cluster.compute();
            cluster.log.note(
                Level::Ok,
                format!(
                    "{KEY_COUNT} keys across {} partitions · {}",
                    cluster.partitions,
                    cluster.strategy.label()
                ),
            );
            cluster
        }

        fn rebuild_points(&mut self) {
            self.points.clear();
            if self.strategy != Strategy::Ring {
                return;
            }
            for partition in 0..self.partitions {
                for replica in 0..self.vnodes {
                    let label = format!("partition-{partition}#{replica}");
                    self.points.push((murmur2(label.as_bytes()), partition));
                }
            }
            self.points.sort_unstable();
        }

        fn bucket_for(&self, hash: u32) -> usize {
            match self.strategy {
                Strategy::Modulo => to_positive(hash) as usize % self.partitions,
                Strategy::Ring => {
                    // First point clockwise at or after the key, wrapping to the
                    // start of the ring — the wrap is what makes it a circle
                    // rather than a line, and dropping it strands the tail keys.
                    match self.points.binary_search_by_key(&hash, |(h, _)| *h) {
                        Ok(i) => self.points[i].1,
                        Err(i) if i < self.points.len() => self.points[i].1,
                        Err(_) => self.points[0].1,
                    }
                }
            }
        }

        fn compute(&self) -> Vec<usize> {
            self.key_hashes
                .iter()
                .map(|h| self.bucket_for(*h))
                .collect()
        }

        /// Recompute the assignment and record what moved. Every mutation goes
        /// through here so the relocation count can never disagree with the
        /// assignment it describes.
        fn apply(&mut self, what: String) {
            self.rebuild_points();
            let next = self.compute();
            self.relocated = self
                .assignment
                .iter()
                .zip(next.iter())
                .enumerate()
                .filter(|(_, (before, after))| before != after)
                .map(|(i, _)| i)
                .collect();
            self.assignment = next;

            let moved = self.relocated.len();
            let pct = moved as f32 / KEY_COUNT as f32 * 100.0;
            let floor = self.balanced_share_pct();
            let level = if moved == 0 {
                Level::Ok
            } else if pct > floor * 2.0 {
                Level::Fail
            } else {
                Level::Held
            };
            self.log.note(
                level,
                format!(
                    "{what} · {moved}/{KEY_COUNT} keys relocated ({pct:.0}%) · a balanced scheme moves about {floor:.0}%",
                ),
            );
        }

        /// The share of keys a *balanced* scheme moves when growing to `n`
        /// buckets: the new bucket should end up holding roughly `1/n` of them,
        /// and they have to come from somewhere. Reporting a relocation count
        /// without it invites the reader to think zero was achievable.
        ///
        /// Deliberately not called a floor. Relocating less than this is not a
        /// better result — it means the new bucket is under-filled, which the
        /// hottest/coldest ratio is there to expose. A ring with few virtual
        /// nodes does exactly that.
        pub(crate) fn balanced_share_pct(&self) -> f32 {
            let to = self.partitions.max(1);
            let from = self.previous_partitions.max(1);
            if to == from {
                return 0.0;
            }
            100.0 / to.max(from) as f32
        }

        pub(crate) fn add_partition(&mut self) {
            if self.partitions >= MAX_PARTITIONS {
                self.log.note(
                    Level::Held,
                    format!("already at {MAX_PARTITIONS} partitions"),
                );
                return;
            }
            self.previous_partitions = self.partitions;
            self.partitions += 1;
            let what = format!(
                "partitions {} → {}",
                self.previous_partitions, self.partitions
            );
            self.apply(what);
        }

        pub(crate) fn remove_partition(&mut self) {
            if self.partitions <= MIN_PARTITIONS {
                self.log.note(
                    Level::Held,
                    format!("already at {MIN_PARTITIONS} partitions"),
                );
                return;
            }
            self.previous_partitions = self.partitions;
            self.partitions -= 1;
            let what = format!(
                "partitions {} → {}",
                self.previous_partitions, self.partitions
            );
            self.apply(what);
        }

        pub(crate) fn set_strategy(&mut self, strategy: Strategy) {
            if self.strategy == strategy {
                return;
            }
            self.strategy = strategy;
            self.previous_partitions = self.partitions;
            self.apply(format!("strategy → {}", strategy.label()));
        }

        /// Cycle the virtual-node count. Only the ring uses it; on modulo it is
        /// recorded but changes nothing, which is itself worth seeing.
        pub(crate) fn cycle_vnodes(&mut self) {
            let current = VNODE_CHOICES
                .iter()
                .position(|v| *v == self.vnodes)
                .unwrap_or(0);
            self.vnodes = VNODE_CHOICES[(current + 1) % VNODE_CHOICES.len()];
            self.previous_partitions = self.partitions;
            self.apply(format!("virtual nodes → {} per partition", self.vnodes));
        }

        pub(crate) fn reset(&mut self) {
            *self = Self::new();
        }

        /// Keys per partition.
        pub(crate) fn load(&self) -> Vec<usize> {
            let mut load = vec![0usize; self.partitions];
            for partition in &self.assignment {
                if let Some(slot) = load.get_mut(*partition) {
                    *slot += 1;
                }
            }
            load
        }

        /// Hottest partition divided by coldest. 1.0 is perfect balance; a naive
        /// ring can reach 3 or more, which is why virtual nodes exist.
        pub(crate) fn spread(&self) -> f32 {
            let load = self.load();
            let max = load.iter().copied().max().unwrap_or(0) as f32;
            let min = load.iter().copied().min().unwrap_or(0) as f32;
            if min == 0.0 {
                return f32::INFINITY;
            }
            max / min
        }

        pub(crate) fn relocated_pct(&self) -> f32 {
            self.relocated.len() as f32 / KEY_COUNT as f32 * 100.0
        }

        pub(crate) fn was_relocated(&self, key: usize) -> bool {
            self.relocated.contains(&key)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn relocation_adding_one(strategy: Strategy, vnodes: usize) -> f32 {
            let mut c = Cluster::new();
            c.vnodes = vnodes;
            c.set_strategy(strategy);
            c.relocated = Vec::new();
            c.add_partition();
            c.relocated_pct()
        }

        #[test]
        fn murmur2_is_deterministic_and_sensitive() {
            assert_eq!(murmur2(b"vehicle-0001"), murmur2(b"vehicle-0001"));
            assert_ne!(murmur2(b"vehicle-0001"), murmur2(b"vehicle-0002"));
            // Every tail length must be exercised by the folding branches.
            let lens: Vec<u32> = ["a", "ab", "abc", "abcd", "abcde"]
                .iter()
                .map(|s| murmur2(s.as_bytes()))
                .collect();
            let unique: std::collections::HashSet<_> = lens.iter().collect();
            assert_eq!(unique.len(), lens.len(), "tail folding collapsed inputs");
            assert_ne!(murmur2(b""), 0);
        }

        #[test]
        fn a_fresh_cluster_assigns_every_key_to_a_real_partition() {
            let c = Cluster::new();
            assert_eq!(c.assignment.len(), KEY_COUNT);
            assert!(c.assignment.iter().all(|p| *p < c.partitions));
            assert_eq!(c.load().iter().sum::<usize>(), KEY_COUNT);
            assert!(c.relocated.is_empty());
            assert_eq!(c, Cluster::default());
        }

        #[test]
        fn both_strategies_stay_total_across_every_partition_count() {
            for strategy in [Strategy::Modulo, Strategy::Ring] {
                let mut c = Cluster::new();
                c.set_strategy(strategy);
                while c.partitions < MAX_PARTITIONS {
                    c.add_partition();
                    assert!(
                        c.assignment.iter().all(|p| *p < c.partitions),
                        "{strategy:?} produced an out-of-range partition"
                    );
                    assert_eq!(c.load().iter().sum::<usize>(), KEY_COUNT);
                }
                while c.partitions > MIN_PARTITIONS {
                    c.remove_partition();
                    assert!(c.assignment.iter().all(|p| *p < c.partitions));
                }
            }
        }

        #[test]
        fn partition_count_is_bounded_in_both_directions() {
            let mut c = Cluster::new();
            for _ in 0..40 {
                c.add_partition();
            }
            assert_eq!(c.partitions, MAX_PARTITIONS);
            for _ in 0..40 {
                c.remove_partition();
            }
            assert_eq!(c.partitions, MIN_PARTITIONS);
        }

        /// The claim the instrument exists to make. Modulo relocates most of the
        /// keyspace on a partition change; the ring stays near the floor it
        /// cannot avoid. Any mutation that swaps the assignment functions, or
        /// drops the ring's wrap-around, collapses this gap.
        #[test]
        fn invariant_the_ring_relocates_far_less_than_modulo() {
            let modulo = relocation_adding_one(Strategy::Modulo, START_VNODES);
            let ring = relocation_adding_one(Strategy::Ring, START_VNODES);
            assert!(
                modulo > 50.0,
                "modulo should relocate most of the keyspace, got {modulo:.0}%"
            );
            assert!(
                ring < modulo / 2.0,
                "ring {ring:.0}% should be far below modulo {modulo:.0}%"
            );
        }

        /// And it stays near the balanced share rather than merely beating
        /// modulo — a scheme that moved half the keys would also pass the test
        /// above.
        #[test]
        fn invariant_the_ring_stays_close_to_the_balanced_share() {
            let mut c = Cluster::new();
            c.set_strategy(Strategy::Ring);
            c.add_partition();
            let floor = c.balanced_share_pct();
            assert!(floor > 0.0);
            assert!(
                c.relocated_pct() <= floor * 2.5,
                "relocated {:.0}% against a {floor:.0}% balanced share",
                c.relocated_pct()
            );
        }

        /// Virtual nodes are the difference between the textbook ring and one
        /// that balances. With a single point per partition the ring is lumpy.
        #[test]
        fn invariant_virtual_nodes_reduce_load_spread() {
            let mut lumpy = Cluster::new();
            lumpy.set_strategy(Strategy::Ring);
            lumpy.vnodes = 1;
            lumpy.set_strategy(Strategy::Modulo);
            lumpy.set_strategy(Strategy::Ring);

            let mut smooth = Cluster::new();
            smooth.vnodes = 128;
            smooth.set_strategy(Strategy::Ring);

            assert!(
                smooth.spread() < lumpy.spread(),
                "128 vnodes spread {:.2} should beat 1 vnode spread {:.2}",
                smooth.spread(),
                lumpy.spread()
            );
        }

        /// Consistent hashing's other property: membership changes are
        /// reversible. Removing a partition and adding it back restores the
        /// original assignment exactly. Modulo happens to share this, but for
        /// the ring it is structural rather than arithmetic coincidence.
        #[test]
        fn invariant_ring_membership_changes_are_reversible() {
            let mut c = Cluster::new();
            c.set_strategy(Strategy::Ring);
            let before = c.assignment.clone();
            c.add_partition();
            assert_ne!(c.assignment, before);
            c.remove_partition();
            assert_eq!(
                c.assignment, before,
                "returning to the same membership must restore the same mapping"
            );
        }

        #[test]
        fn the_balanced_share_is_zero_when_membership_does_not_change() {
            let mut c = Cluster::new();
            assert_eq!(c.balanced_share_pct(), 0.0);
            c.set_strategy(Strategy::Ring);
            assert_eq!(
                c.balanced_share_pct(),
                0.0,
                "a strategy switch moves keys but adds no capacity"
            );
        }

        #[test]
        fn switching_to_the_same_strategy_changes_nothing() {
            let mut c = Cluster::new();
            let before = c.clone();
            c.set_strategy(Strategy::Modulo);
            assert_eq!(c, before);
        }

        #[test]
        fn cycling_vnodes_walks_the_choices_and_wraps() {
            let mut c = Cluster::new();
            c.set_strategy(Strategy::Ring);
            let start = VNODE_CHOICES.iter().position(|v| *v == c.vnodes).unwrap();
            for step in 1..=VNODE_CHOICES.len() {
                c.cycle_vnodes();
                assert_eq!(
                    c.vnodes,
                    VNODE_CHOICES[(start + step) % VNODE_CHOICES.len()]
                );
            }
            assert_eq!(c.vnodes, START_VNODES, "a full cycle returns to the start");
        }

        #[test]
        fn relocated_keys_are_exactly_those_whose_partition_changed() {
            let mut c = Cluster::new();
            let before = c.assignment.clone();
            c.add_partition();
            for key in 0..KEY_COUNT {
                assert_eq!(
                    c.was_relocated(key),
                    before[key] != c.assignment[key],
                    "key {key} misreported"
                );
            }
        }

        #[test]
        fn spread_is_finite_while_every_partition_holds_keys() {
            let c = Cluster::new();
            assert!(c.spread().is_finite());
            assert!(c.spread() >= 1.0);
        }

        #[test]
        fn reset_restores_the_starting_cluster() {
            let mut c = Cluster::new();
            c.set_strategy(Strategy::Ring);
            c.add_partition();
            c.cycle_vnodes();
            c.reset();
            assert_eq!(c, Cluster::new());
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) mod view {
    use super::logic::{Cluster, KEY_COUNT, MAX_PARTITIONS, MIN_PARTITIONS, Strategy};
    use leptos::prelude::*;

    #[component]
    pub(crate) fn Partitioning() -> impl IntoView {
        let c = RwSignal::new(Cluster::new());

        view! {
            <div class="inst" role="group" aria-label="Key partitioning instrument">
                <div class="inst-head">
                    <span>"TOPIC/PARTITIONING"</span>
                    <span class="inst-anchor">
                        {move || c.with(|c| c.strategy.label().to_uppercase())}
                    </span>
                </div>

                <div class="inst-controls">
                    <button
                        type="button"
                        class:active=move || c.with(|c| c.strategy == Strategy::Modulo)
                        on:click=move |_| c.update(|c| c.set_strategy(Strategy::Modulo))
                    >"MURMUR2 % N"</button>
                    <button
                        type="button"
                        class:active=move || c.with(|c| c.strategy == Strategy::Ring)
                        on:click=move |_| c.update(|c| c.set_strategy(Strategy::Ring))
                    >"HASH RING"</button>
                </div>

                <div class="inst-bars" role="img"
                    aria-label=move || c.with(|c| format!(
                        "{} partitions holding {:?} keys", c.partitions, c.load()))>
                    {move || c.with(|c| {
                        let load = c.load();
                        let peak = load.iter().copied().max().unwrap_or(1).max(1);
                        load.into_iter().enumerate().map(|(p, n)| {
                            let pct = n as f32 / peak as f32 * 100.0;
                            view! {
                                <div class="bar">
                                    <span class="bar-fill" style:height=format!("{pct}%")></span>
                                    <b>{n}</b>
                                    <i>{format!("p{p}")}</i>
                                </div>
                            }
                        }).collect_view()
                    })}
                </div>

                <div class="inst-keys" role="img"
                    aria-label=move || c.with(|c| format!("{} of {KEY_COUNT} keys relocated", c.relocated.len()))>
                    {move || c.with(|c| (0..KEY_COUNT).map(|k| {
                        let class = if c.was_relocated(k) { "key moved" } else { "key" };
                        view! { <span class=class></span> }
                    }).collect_view())}
                </div>
                <div class="inst-legend">
                    <span><i class="key"></i>"KEY, WHERE IT WAS"</span>
                    <span><i class="key moved"></i>"RELOCATED BY THE LAST CHANGE"</span>
                </div>

                <div class="inst-readout">
                    <div>
                        <span>"PARTITIONS"</span>
                        <strong>{move || c.with(|c| c.partitions)}</strong>
                    </div>
                    <div>
                        <span>"BALANCED SHARE"</span>
                        <strong>{move || c.with(|c| format!("{:.0}%", c.balanced_share_pct()))}</strong>
                    </div>
                    <div>
                        <span>"HOTTEST / COLDEST"</span>
                        <strong>{move || c.with(|c| format!("{:.2}×", c.spread()))}</strong>
                    </div>
                    <div>
                        <span>"VIRTUAL NODES"</span>
                        <strong>{move || c.with(|c| c.vnodes)}</strong>
                    </div>
                    <div class="inst-cost">
                        <span>"KEYS RELOCATED"</span>
                        <strong class:hot=move || c.with(|c| c.relocated_pct() > c.balanced_share_pct() * 2.0)>
                            {move || c.with(|c| format!("{:.0}%", c.relocated_pct()))}
                        </strong>
                    </div>
                </div>

                <Show when=move || c.with(|c| c.strategy == Strategy::Modulo && !c.relocated.is_empty())>
                    <p class="inst-pending">
                        "Every relocated key now has messages behind two partitions with no ordering
                         relationship between them. Kafka guarantees order within a partition, so for
                         these keys the guarantee is simply gone — and nothing in the cluster reports it."
                    </p>
                </Show>

                <div class="inst-actions">
                    <button type="button"
                        prop:disabled=move || c.with(|c| c.partitions >= MAX_PARTITIONS)
                        on:click=move |_| c.update(|c| c.add_partition())>"ADD PARTITION"</button>
                    <button type="button"
                        prop:disabled=move || c.with(|c| c.partitions <= MIN_PARTITIONS)
                        on:click=move |_| c.update(|c| c.remove_partition())>"REMOVE PARTITION"</button>
                    <button type="button" on:click=move |_| c.update(|c| c.cycle_vnodes())>
                        "CYCLE VIRTUAL NODES"
                    </button>
                    <button type="button" on:click=move |_| c.update(|c| c.reset())>"RESET"</button>
                </div>

                <p class="inst-note">
                    {format!("{KEY_COUNT} keys named vehicle-0000… hashed with Kafka's murmur2 (seed 0x9747b28c). \
                              1/n is the share a balanced scheme moves to fill a new partition. Moving far less is not a win — \
                              it means the new partition is under-filled, which the hottest/coldest ratio exposes.")}
                </p>

                <ol class="inst-log">
                    {move || c.with(|c| c.log.tail().iter().map(|line| {
                        view! { <li class=line.level.class()>{line.text.clone()}</li> }
                    }).collect_view())}
                </ol>
            </div>
        }
    }
}
