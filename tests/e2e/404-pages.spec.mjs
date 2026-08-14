import { expect, test } from "@playwright/test";

test("the official fallback is a focused terrain recovery page", async ({ page }) => {
  await page.goto("/404.html?from=broken#route", { waitUntil: "domcontentloaded" });

  await expect(page.getByRole("heading", { level: 1, name: "Off the map." })).toBeVisible();
  await expect(page.locator(".coordinate-contours")).toBeVisible();
  await expect(page.getByRole("link", { name: "Recenter home" })).toBeVisible();
  await expect(page.locator(".error-button")).toHaveCount(1);
  await expect(page.getByText(/concept/i)).toHaveCount(0);
  await expect(page.locator("[data-request-path]")).toHaveCount(0);

  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("the contour map covers every viewport without losing detail", async ({ page }) => {
  await page.goto("/404.html", { waitUntil: "domcontentloaded" });

  const geometry = await page.locator(".coordinate-contours").evaluate((map) => {
    const mapBounds = map.getBoundingClientRect();
    const layoutBounds = map.parentElement.getBoundingClientRect();
    return {
      mapWidth: mapBounds.width,
      mapHeight: mapBounds.height,
      layoutWidth: layoutBounds.width,
      layoutHeight: layoutBounds.height,
      mapLeft: mapBounds.left,
      mapRight: mapBounds.right,
      layoutLeft: layoutBounds.left,
      layoutRight: layoutBounds.right,
      paths: map.querySelectorAll("path").length,
    };
  });

  expect(geometry.mapWidth).toBeGreaterThanOrEqual(geometry.layoutWidth);
  expect(Math.abs(geometry.mapHeight - geometry.layoutHeight)).toBeLessThanOrEqual(1);
  expect(geometry.mapLeft).toBeLessThanOrEqual(geometry.layoutLeft);
  expect(geometry.mapRight).toBeGreaterThanOrEqual(geometry.layoutRight);
  expect(geometry.paths).toBeGreaterThanOrEqual(12);
});
