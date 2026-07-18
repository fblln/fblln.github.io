import type { Meta, StoryObj } from "@storybook/react";
import { Button } from "./Button";

const meta: Meta<typeof Button> = {
  title: "Core UI/Button",
  component: Button,
  args: { children: "Inspect systems ↓" },
  argTypes: { variant: { control: "inline-radio", options: ["outline", "solid"] } },
};
export default meta;

type Story = StoryObj<typeof Button>;

export const Outline: Story = { args: { variant: "outline" } };
export const Solid: Story = { args: { variant: "solid" } };
export const AsLink: Story = {
  args: { variant: "solid", href: "#", children: "Start a conversation ↗" },
};
export const Disabled: Story = { args: { disabled: true } };
