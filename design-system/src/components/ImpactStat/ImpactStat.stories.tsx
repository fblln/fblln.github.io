import type { Meta, StoryObj } from "@storybook/react";
import { ImpactStat } from "./ImpactStat";

const meta: Meta<typeof ImpactStat> = {
  title: "Site Patterns/ImpactStat",
  component: ImpactStat,
  args: { value: "10M+", label: "connected vehicles supported" },
};
export default meta;

type Story = StoryObj<typeof ImpactStat>;

export const Default: Story = {};
export const Grid: Story = {
  parameters: { layout: "fullscreen" },
  render: () => (
    <div style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)" }}>
      <ImpactStat value="10M+" label="connected vehicles supported" />
      <ImpactStat value="100+" label="B2B and B2C APIs shaped" />
      <ImpactStat value="0→20+" label="engineering organization growth" />
      <ImpactStat value="100K+" label="IoT devices operated" />
    </div>
  ),
};
