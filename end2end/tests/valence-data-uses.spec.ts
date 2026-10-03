import { test, expect } from "@playwright/test";
import { seedVerifiedUser, signIn } from "./support/auth";
import { gotoHydrated } from "./support/hydration";

const hasSeedToken = Boolean(process.env.UF_E2E_SEED_TOKEN?.trim());

/** Purpose text that only exists in valence-uf-app's e2e fixtures. */
const FIXTURE_PURPOSES = [
  "User access in data_use_catalog_fixtures",
  "Fixture this data access",
  "E2E_TEST_ONLY_PURPOSE",
] as const;

test.describe("pw-l5-valence-data-uses", () => {
  test("pw-l5-valence-data-uses-anonymous-sad", async ({ page }) => {
    await gotoHydrated(page, "/valence/schema/counter");
    await expect(page.getByTestId("auth-required-empty-state")).toBeAttached({
      timeout: 60_000,
    });
    await expect(page.getByTestId("valence-schema-data-uses")).toHaveCount(0);
    await expect(page.getByText(/counter-app-worker · /)).toHaveCount(0);
  });

  test.describe("signed in", () => {
    test.beforeEach(() => {
      test.skip(!hasSeedToken, "UF_E2E_SEED_TOKEN required to sign in");
    });

    test("pw-l5-valence-data-uses-counter-happy", async ({ page, request }) => {
      const credentials = await seedVerifiedUser(request);
      await signIn(page, credentials, "/valence/schema/counter");
      const panel = page.getByTestId("valence-schema-data-uses");
      await expect(panel).toBeVisible({ timeout: 60_000 });
      await expect(page.getByTestId("valence-data-uses-catalog-missing")).toHaveCount(0);
      await expect(panel.getByText("No declared uses")).toHaveCount(0);
      const rows = panel.getByTestId("valence-data-use-row");
      await expect(rows.first()).toBeVisible({ timeout: 60_000 });
      await expect(
        rows.filter({ hasText: /counter-app-worker · / }).first(),
      ).toBeVisible();
      await expect(
        rows.first().getByRole("link", { name: "View source" }),
      ).toHaveAttribute("href", /^https:\/\/github\.com\/[^/]+\/[^/]+\/blob\/main\/[^/]/);
    });

    test("pw-l5-valence-data-uses-no-fixture-leak", async ({ page, request }) => {
      const credentials = await seedVerifiedUser(request);
      await signIn(page, credentials, "/valence/schema/user");
      await expect(page.getByTestId("valence-schema-data-uses")).toBeVisible({
        timeout: 60_000,
      });
      for (const purpose of FIXTURE_PURPOSES) {
        await expect(page.getByText(purpose)).toHaveCount(0);
      }

      await gotoHydrated(page, "/valence/unscoped-uses");
      await expect(page.getByTestId("valence-unscoped-data-uses")).toBeVisible({
        timeout: 60_000,
      });
      for (const purpose of FIXTURE_PURPOSES) {
        await expect(page.getByText(purpose)).toHaveCount(0);
      }
      await expect(page.getByText(/valence-uf-app-e2e · /)).toHaveCount(0);
    });
  });
});
