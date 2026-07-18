import type { Meta, StoryObj } from "@storybook/react";
import { ArticleHeader } from "./ArticleHeader";

const meta: Meta<typeof ArticleHeader> = {
  title: "Article/ArticleHeader",
  component: ArticleHeader,
  args: {
    eyebrow: "2026-07-17 · 4 min read",
    title: "Systems That Survive Contact With Production",
    tags: ["Systems", "Reliability", "Rust"],
  },
};
export default meta;

export const Default: StoryObj<typeof ArticleHeader> = {};
