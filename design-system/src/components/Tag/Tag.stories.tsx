import type { Meta, StoryObj } from "@storybook/react";
import { Tag } from "./Tag";

const meta: Meta<typeof Tag> = {
  title: "Core UI/Tag",
  component: Tag,
  args: { children: "Rust" },
};
export default meta;

type Story = StoryObj<typeof Tag>;

export const Default: Story = {};
export const Row: Story = {
  render: () => (
    <div style={{ display: "flex", gap: "0.35rem", flexWrap: "wrap" }}>
      {["Rust", "WASM", "Terrain", "GPX"].map((t) => (
        <Tag key={t}>{t}</Tag>
      ))}
    </div>
  ),
};
