import type { Meta, StoryObj } from "@storybook/react";
import { TableOfContents } from "./TableOfContents";

const meta: Meta<typeof TableOfContents> = {
  title: "Article/TableOfContents",
  component: TableOfContents,
  args: {
    entries: [
      { level: 2, text: "Measure the real system", href: "#measure" },
      { level: 2, text: "Make evidence queryable", href: "#evidence" },
      { level: 3, text: "A quick comparison", href: "#comparison" },
      { level: 2, text: "Ship the smallest solid thing", href: "#ship" },
    ],
  },
};
export default meta;

export const Default: StoryObj<typeof TableOfContents> = {};
