import type { Meta, StoryObj } from "@storybook/react";
import { ProjectRow } from "./ProjectRow";

const meta: Meta<typeof ProjectRow> = {
  title: "Site Patterns/ProjectRow",
  component: ProjectRow,
  parameters: { layout: "fullscreen" },
  args: {
    number: 3,
    name: "Ridgeline",
    stack: "RUST · WASM · GDAL · THREE.JS",
    summary: "Transform a GPX route and elevation data into an interactive terrain artifact.",
    tags: ["Rust", "WASM", "Terrain", "GPX"],
  },
};
export default meta;

type Story = StoryObj<typeof ProjectRow>;

export const Default: Story = {};
export const Active: Story = { args: { active: true } };
export const List: Story = {
  render: () => (
    <div style={{ borderTop: "2px solid var(--ink)" }}>
      <ProjectRow number={1} name="Race Telemetry Workbench" stack=".NET · TIMESCALEDB · MCP" active />
      <ProjectRow number={2} name="Lithograph" stack="RUST · TREE-SITTER · MCP" />
      <ProjectRow number={3} name="Ridgeline" stack="RUST · WASM · GDAL" />
    </div>
  ),
};
