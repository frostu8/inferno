import { test, expect } from "@playwright/test";

test("sanity check", async ({ page }) => {
  await page.goto("http://localhost:4000/~/Index");
})

test("check sidebar", async ({ page }) => {
  await page.goto("http://localhost:4000/~/Index");

  await expect(page.locator("#sidebar h1")).toHaveText("~/inferno");
});
