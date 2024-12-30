import { cli, test } from "./utils";
import { expect } from "@playwright/test";

test.describe(
    "examples/basic/inject.yml",
    {
        annotation: {
            type: cli({
                args: ["-i", "examples/basic/inject.yml"],
            }),
            description: "",
        },
    },
    () => {
        test("inject bslive:connector", async ({ request, bs }) => {
            const response = await request.get(bs.path("/"), {
                headers: {
                    accept: "text/html",
                },
            });
            const body = await response.body();
            expect(body.toString()).toContain(
                `<script type="module" src="/__bs_js"></script>`,
            );

            {
                const response = await request.get(bs.path("/form.html"), {
                    headers: {
                        accept: "text/html",
                    },
                });
                const body = await response.body();
                expect(body.toString()).not.toContain(
                    `<script type="module" src="/__bs_js"></script>`,
                );
            }
        });
        test("injects with bslive:js-connector query param", async ({
            page,
            bs,
        }) => {
            await page.goto(bs.named("no-inject", "/"));
            const waiter = page.waitForRequest((req) =>
                new URL(req.url()).pathname.startsWith("/__bs_js"),
            );
            await page.addScriptTag({
                url: bs.named(
                    "no-inject",
                    "/script2.js?bslive.inject=bslive:js-connector",
                ),
                type: "module",
            });
            await waiter;
        });
    },
);
