import type { Meta, StoryObj } from "@storybook/react";
import { DiagnosticsPanel } from "./DiagnosticsPanel";

const meta: Meta<typeof DiagnosticsPanel> = {
  title: "Site Patterns/DiagnosticsPanel",
  component: DiagnosticsPanel,
  parameters: { layout: "fullscreen", surface: "ink" },
  args: {
    items: [
      { label: "Application", value: "LEPTOS CSR" },
      { label: "Target", value: "WASM32-UNKNOWN-UNKNOWN" },
      { label: "Boot to WASM entry", value: "10.0 ms" },
      { label: "WASM received", value: "64 KiB" },
    ],
  },
};
export default meta;

export const Default: StoryObj<typeof DiagnosticsPanel> = {};
