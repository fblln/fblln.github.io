import { defineConfig } from "tsup";

// esbuild bundles the components AND every CSS they import into dist/index.css
// (tokens + global come first, via src/index.ts). That single file is exposed
// as "@fblln/design-system/styles.css".
export default defineConfig({
  entry: ["src/index.ts"],
  format: ["esm"],
  dts: true,
  clean: true,
  external: ["react", "react-dom", "react/jsx-runtime"],
});
