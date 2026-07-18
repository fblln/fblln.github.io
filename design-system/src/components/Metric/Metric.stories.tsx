import type { Meta, StoryObj } from "@storybook/react";
import { Metric } from "./Metric";

const meta: Meta<typeof Metric> = {
  title: "Site Patterns/Metric",
  component: Metric,
  parameters: { surface: "ink" },
  args: { value: "42.7×", label: "raw telemetry compression" },
};
export default meta;

export const Default: StoryObj<typeof Metric> = {};
