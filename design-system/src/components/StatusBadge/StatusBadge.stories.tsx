import type { Meta, StoryObj } from "@storybook/react";
import { StatusBadge } from "./StatusBadge";

const meta: Meta<typeof StatusBadge> = {
  title: "Core UI/StatusBadge",
  component: StatusBadge,
  args: { children: "WASM/ACTIVE" },
};
export default meta;

type Story = StoryObj<typeof StatusBadge>;

export const Healthy: Story = { args: { active: false } };
export const Active: Story = { args: { active: true } };
