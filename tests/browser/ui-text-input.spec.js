import { expect, test } from "@playwright/test";

test("GPE.UI text input bridges DOM edits and submit back to Rust", async ({ page }) => {
  await page.goto("/tests/browser/ui-text-input.html");

  await expect
    .poll(
      () => page.evaluate(() => globalThis.__gpeTextInputState?.phase ?? "missing"),
      { timeout: 15_000 },
    )
    .toMatch(/^(initialized|winit-handoff)$/);

  // Startup errors belong to winit-smoke. This probe waits for evidence that
  // the GPE game loop actually reached TextInputProbe::update before testing
  // the DOM <-> Rust contract.
  await expect
    .poll(
      () =>
        page.evaluate(() =>
          globalThis.__gpeTextInputState?.snapshot
            ? globalThis.__gpeTextInputState.snapshot()
            : "",
        ),
      { timeout: 30_000 },
    )
    .toContain("search=");

  const inputs = page.locator("[data-gpe-ui-text-input='1']");
  await expect(inputs).toHaveCount(2, { timeout: 10_000 });

  const search = inputs.nth(0);
  const player = inputs.nth(1);

  await search.click();
  await search.fill("minoku");
  await expect
    .poll(() =>
      page.evaluate(() => globalThis.__gpeTextInputState.snapshot()),
    )
    .toContain("search=minoku");

  await search.press("Enter");
  await expect
    .poll(() =>
      page.evaluate(() => globalThis.__gpeTextInputState.snapshot()),
    )
    .toContain("search_submits=1");

  await player.click();
  await player.fill("Gotoo");
  await expect
    .poll(() =>
      page.evaluate(() => globalThis.__gpeTextInputState.snapshot()),
    )
    .toContain("player=Gotoo");

  await player.press("Enter");
  await expect
    .poll(() =>
      page.evaluate(() => globalThis.__gpeTextInputState.snapshot()),
    )
    .toContain("player_submits=1");
});
