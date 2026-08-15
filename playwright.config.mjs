import { defineConfig, devices } from "@playwright/test";

/* The 404 surface is intentionally independent of Trunk, so the browser suite
   serves authored files directly. Release-build checks separately guarantee
   those same files reach dist/; splitting the seams makes failures specific. */
export default defineConfig({
  testDir: "./tests/e2e",
  timeout: 120_000,
  fullyParallel: false,
  workers: 1,
  use: {
    baseURL: "http://127.0.0.1:4173",
    trace: "retain-on-failure",
  },
  projects: [
    { name: "desktop-chromium", use: { ...devices["Desktop Chrome"] } },
    { name: "mobile-chromium", use: { ...devices["Pixel 7"] } },
  ],
  webServer: {
    command: "python3 -m http.server 4173 --bind 127.0.0.1 --directory dist",
    url: "http://127.0.0.1:4173/404.html",
    reuseExistingServer: true,
  },
});
