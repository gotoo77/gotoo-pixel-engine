import { expect, test } from "@playwright/test";

test("real wasm-bindgen + winit startup does not become a false fatal error", async ({ page }) => {
  const pageErrors = [];
  page.on("pageerror", (error) => pageErrors.push(error.message));

  await page.goto("/tests/browser/winit-smoke.html");

  await expect
    .poll(
      () => page.evaluate(() => globalThis.__gpeSmokeState?.phase ?? "missing"),
      { timeout: 15_000 },
    )
    .toMatch(/^(initialized|winit-handoff)$/);

  const state = await page.evaluate(() => globalThis.__gpeSmokeState);
  expect(state.webgpu).toBe(true);
  expect(["initialized", "winit-handoff"]).toContain(state.phase);

  if (state.phase === "winit-handoff") {
    expect(state.message).toContain("Using exceptions for control flow");
    expect(state.message).toContain("This isn't actually an error");
  }

  await expect(page.locator("canvas")).toBeVisible();
  expect(pageErrors).toEqual([]);
});
