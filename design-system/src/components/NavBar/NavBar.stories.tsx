import type { Meta, StoryObj } from "@storybook/react";
import { NavBar } from "./NavBar";
import { StatusBadge } from "../StatusBadge/StatusBadge";

const meta: Meta<typeof NavBar> = {
  title: "Core UI/NavBar",
  component: NavBar,
  parameters: { layout: "fullscreen" },
  args: {
    wordmark: "FE/26",
    links: [
      { label: "Work", href: "#work" },
      { label: "Impact", href: "#impact" },
      { label: "Experience", href: "#experience" },
      { label: "Writing", href: "/articles/" },
    ],
    action: <StatusBadge active>WASM/ACTIVE</StatusBadge>,
  },
};
export default meta;

export const Default: StoryObj<typeof NavBar> = {};
