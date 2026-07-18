import type { Meta, StoryObj } from "@storybook/react";
import { Prose } from "./Prose";

const meta: Meta<typeof Prose> = {
  title: "Article/Prose",
  component: Prose,
};
export default meta;

export const Default: StoryObj<typeof Prose> = {
  render: () => (
    <Prose>
      <h2>Measure the real system</h2>
      <p>
        Production distributions, traces, and representative data beat architectural theatre.
        Before optimizing, look at what actually happens.
      </p>
      <p>
        A single <code>p99</code> on real data will reshape more decisions than a week of
        whiteboarding.
      </p>
      <blockquote>Numbers are cheap; opinions are expensive.</blockquote>
      <ul>
        <li>Broad demo — low trust, low leverage.</li>
        <li>Narrow and solid — high trust, high leverage.</li>
      </ul>
    </Prose>
  ),
};
