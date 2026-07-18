import type { Meta, StoryObj } from "@storybook/react";
import { SiteFooter } from "./SiteFooter";

const meta: Meta<typeof SiteFooter> = {
  title: "Core UI/SiteFooter",
  component: SiteFooter,
  parameters: { layout: "fullscreen", surface: "ink" },
};
export default meta;

export const Default: StoryObj<typeof SiteFooter> = {
  render: () => (
    <SiteFooter>
      <span>© 2026 FABIO ELLENA</span>
      <a href="/articles/">WRITING ↗</a>
      <span>FULL RUST · LEPTOS · WASM32</span>
    </SiteFooter>
  ),
};
