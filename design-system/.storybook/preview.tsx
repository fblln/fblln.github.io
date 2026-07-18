import type { Preview } from "@storybook/react";
import React from "react";
import "../src/styles/tokens.css";
import "../src/styles/global.css";

const preview: Preview = {
  parameters: {
    controls: { expanded: true },
    options: {
      storySort: {
        order: ["Foundations", "Core UI", "Site Patterns", "Article"],
      },
    },
  },
  decorators: [
    (Story, ctx) => {
      const dark = ctx.parameters.surface === "ink";
      return (
        <div
          style={{
            padding: "2rem",
            background: dark ? "#0a0a0a" : "#f2f0e9",
            color: dark ? "#f2f0e9" : "#0a0a0a",
            minHeight: "60vh",
          }}
        >
          <Story />
        </div>
      );
    },
  ],
};

export default preview;
