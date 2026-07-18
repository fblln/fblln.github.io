import type { Meta, StoryObj } from "@storybook/react";
import { Callout } from "./Callout";

const meta: Meta<typeof Callout> = {
  title: "Article/Callout",
  component: Callout,
  args: {
    children:
      "A narrow production-grade loop creates more leverage than a broad demo that cannot be trusted.",
  },
};
export default meta;

export const Default: StoryObj<typeof Callout> = {};
