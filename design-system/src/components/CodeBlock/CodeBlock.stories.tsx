import type { Meta, StoryObj } from "@storybook/react";
import { CodeBlock } from "./CodeBlock";

const meta: Meta<typeof CodeBlock> = {
  title: "Article/CodeBlock",
  component: CodeBlock,
  args: {
    lang: "rust",
    code: `fn p99(latencies: &mut [u64]) -> u64 {
    latencies.sort_unstable();
    let idx = (latencies.len() as f64 * 0.99) as usize;
    latencies[idx.min(latencies.len() - 1)]
}`,
  },
};
export default meta;

export const Default: StoryObj<typeof CodeBlock> = {};
