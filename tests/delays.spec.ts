import { cli, test } from "./utils";
import { expect } from "@playwright/test";

test.describe(
    "examples/basic/delays.yml",
    {
        annotation: {
            type: cli({
                args: ["-i", "examples/basic/delays.yml"],
            }),
            description: "",
        },
    },
    () => {
        test("first delay item", async ({ request, bs }) => {
            const start = Date.now();
            const response = await request.get(bs.path("/"));

            const body = await response.body();
            const diff = Date.now() - start;

            expect(body.toString()).toBe(`first - 200ms delay`);
            expect(diff).toBeGreaterThan(200);
            expect(diff).toBeLessThan(300);
        });
        test("no config-based delay (url param)", async ({ request, bs }) => {
            const start = Date.now();
            const response = await request.get(
                bs.path("/none?bslive.delay.ms=200"),
            );

            const body = await response.body();
            const diff = Date.now() - start;

            expect(body.toString()).toBe(`no config-based delay`);
            expect(diff).toBeGreaterThan(200);
            expect(diff).toBeLessThan(300);
        });
        test("500ms delay", async ({ request, bs }) => {
            const start = Date.now();
            const response = await request.get(bs.path("/500"));

            const body = await response.body();
            const diff = Date.now() - start;

            expect(body.toString()).toBe(`second - 500ms delay`);
            expect(diff).toBeGreaterThan(500);
            expect(diff).toBeLessThan(600);
        });
    },
);
