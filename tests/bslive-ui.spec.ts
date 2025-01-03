import { cli, test } from "./utils";
import { expect } from "@playwright/test";

test.describe(
    "Browsersync bslive 404 UI",
    {
        annotation: {
            type: cli({
                args: ["-i", "examples/basic/headers.yml"],
            }),
        },
    },
    () => {
        test("shows the UI", async ({ page, request, bs }) => {
            await page.goto(bs.path("/__bslive"));
            await page.locator("bs-header").waitFor({ timeout: 1000 });
        });
    },
);

test.describe(
    "Browsersync bslive 404 fallback",
    {
        annotation: {
            type: cli({
                args: ["-i", "examples/basic/fallback.yml"],
            }),
        },
    },
    () => {
        test("shows the UI as fallback on DIR", async ({ page, bs }) => {
            const r = await page.goto(bs.path("/"));
            await page.locator("bs-header").waitFor({ timeout: 1000 });
            expect(r?.status()).toEqual(404);
        });
    },
);
