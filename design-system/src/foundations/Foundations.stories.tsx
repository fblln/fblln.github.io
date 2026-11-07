import type { Meta, StoryObj } from "@storybook/react";

const meta: Meta = {
  title: "Foundations/Tokens",
  parameters: { layout: "fullscreen" },
};
export default meta;

const COLORS: { name: string; token: string; note: string }[] = [
  { name: "Ink", token: "--ink", note: "text / dark surfaces" },
  { name: "Paper", token: "--paper", note: "page background" },
  { name: "Signal", token: "--signal", note: "accent — fills, display type" },
  { name: "Signal text", token: "--signal-text", note: "accent as small text on paper" },
  { name: "Muted", token: "--muted", note: "secondary text" },
  { name: "Code bg", token: "--code-bg", note: "code blocks" },
  { name: "OK", token: "--ok", note: "healthy status" },
];

const TYPE: { label: string; family: string; sample: string; style?: React.CSSProperties }[] = [
  { label: "Display — var(--font-sans)", family: "var(--font-sans)", sample: "Systems that survive production", style: { fontSize: "2.4rem", fontWeight: 800, letterSpacing: "-0.04em" } },
  { label: "Body reading — var(--font-reading)", family: "var(--font-reading)", sample: "Most architecture diagrams are fiction.", style: { fontSize: "1.15rem", lineHeight: 1.7 } },
  { label: "Mono meta — var(--font-mono)", family: "var(--font-mono)", sample: "WASM32-UNKNOWN-UNKNOWN · 64 KIB", style: { fontSize: "0.8rem", letterSpacing: "0.06em", textTransform: "uppercase" } },
];

export const Palette: StoryObj = {
  render: () => (
    <div style={{ padding: "2rem", fontFamily: "var(--font-sans)" }}>
      <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(180px, 1fr))", gap: "1rem" }}>
        {COLORS.map((c) => (
          <div key={c.token} style={{ border: "1px solid var(--line)" }}>
            <div style={{ height: 90, background: `var(${c.token})` }} />
            <div style={{ padding: "0.6rem 0.75rem" }}>
              <strong style={{ display: "block", fontSize: "0.85rem" }}>{c.name}</strong>
              <code style={{ fontFamily: "var(--font-mono)", fontSize: "0.7rem" }}>{c.token}</code>
              <div style={{ color: "var(--muted)", fontSize: "0.7rem", textTransform: "uppercase", letterSpacing: "0.05em", marginTop: 2 }}>{c.note}</div>
            </div>
          </div>
        ))}
      </div>
    </div>
  ),
};

/* The three surfaces the site alternates between. Orange is a surface, not a
   text colour: it goes to large fields, and everything on it follows the rules
   listed in each card. */
const SURFACES: { name: string; bg: string; fg: string; secondary: string; uses: string; rules: string[] }[] = [
  {
    name: "Paper",
    bg: "var(--paper)",
    fg: "var(--ink)",
    secondary: "var(--muted)",
    uses: "Default. Hero, work, stack, experience, interests, Writing.",
    rules: ["Secondary text: --muted", "Small accent text: --signal-text", "Accent hover on links is allowed"],
  },
  {
    name: "Ink",
    bg: "var(--ink)",
    fg: "var(--paper)",
    secondary: "var(--muted-invert)",
    uses: "Principles band, footer, diagnostics panel, instruments.",
    rules: ["Secondary text: --muted-invert", "--signal passes as text here (5.5:1)"],
  },
  {
    name: "Signal",
    bg: "var(--signal)",
    fg: "var(--ink)",
    secondary: "color-mix(in srgb, var(--ink) 78%, transparent)",
    uses: "Education band, contact section, the open project row.",
    rules: [
      "Text: ink (5.5:1); secondary: ink at 78%",
      "Never --muted (1.7:1) or orange text",
      "Links stay ink; hover thickens the underline",
      "Focus and selection use ink",
      "Never two signal surfaces in a row",
    ],
  },
];

export const Surfaces: StoryObj = {
  render: () => (
    <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(240px, 1fr))", fontFamily: "var(--font-sans)" }}>
      {SURFACES.map((s) => (
        <section key={s.name} style={{ background: s.bg, color: s.fg, padding: "2rem 1.5rem", display: "grid", gap: "0.75rem", alignContent: "start" }}>
          <div style={{ fontFamily: "var(--font-mono)", fontSize: "var(--label)", fontWeight: 700, letterSpacing: "0.08em", textTransform: "uppercase" }}>{s.name}</div>
          <div style={{ fontSize: "2rem", fontWeight: 800, letterSpacing: "-0.04em", lineHeight: 0.95 }}>BORING SOFTWARE</div>
          <p style={{ margin: 0, color: s.secondary, lineHeight: 1.45 }}>{s.uses}</p>
          <ul style={{ margin: 0, paddingLeft: "1.1rem", fontFamily: "var(--font-mono)", fontSize: "var(--label)", lineHeight: 1.6 }}>
            {s.rules.map((r) => (
              <li key={r}>{r}</li>
            ))}
          </ul>
        </section>
      ))}
    </div>
  ),
};

export const Typography: StoryObj = {
  render: () => (
    <div style={{ padding: "2rem", display: "grid", gap: "2.5rem" }}>
      {TYPE.map((t) => (
        <div key={t.label}>
          <div style={{ fontFamily: "var(--font-mono)", fontSize: "0.68rem", textTransform: "uppercase", letterSpacing: "0.08em", color: "var(--muted)", marginBottom: "0.5rem" }}>{t.label}</div>
          <div style={{ fontFamily: t.family, ...t.style }}>{t.sample}</div>
        </div>
      ))}
    </div>
  ),
};
