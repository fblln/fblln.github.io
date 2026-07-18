import type { Meta, StoryObj } from "@storybook/react";
import { SectionHeading } from "./SectionHeading";

const meta: Meta<typeof SectionHeading> = {
  title: "Site Patterns/SectionHeading",
  component: SectionHeading,
  parameters: { layout: "fullscreen" },
  args: {
    eyebrow: "SELECTED SYSTEMS",
    title: "THE WORK",
    description:
      "Four public projects. Each one built around real data, explicit constraints, and an inspectable technical argument.",
  },
};
export default meta;

export const Default: StoryObj<typeof SectionHeading> = {};
