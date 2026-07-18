import type { Meta, StoryObj } from "@storybook/react";

const meta: Meta = {
  title: "Foundations/Tokens",
  parameters: { layout: "fullscreen" },
};
export default meta;

const COLORS: { name: string; token: string; note: string }[] = [
  { name: "Ink", token: "--ink", note: "text / dark surfaces" },
  { name: "Paper", token: "--paper", note: "page background" },
  { name: "Signal", token: "--signal", note: "accent" },
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
