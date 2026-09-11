import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./tests/browser",
  testMatch: "**/*.spec.js",
  timeout: 30_000,
  expect: {
    timeout: 15_000,
  },
  use: {
    baseURL: "http://127.0.0.1:4173",
    browserName: "chromium",
    headless: true,
    launchOptions: {
      args: ["--enable-unsafe-webgpu"],
    },
  },
  webServer: {
    command: "python3 -m http.server 4173 --bind 127.0.0.1 --directory .",
    url: "http://127.0.0.1:4173/tests/browser/winit-smoke.html",
    reuseExistingServer: false,
  },
});
