import { cli, installMockHandler, readCalls, test } from "./utils";
import { expect } from "@playwright/test";

test.describe(
    "examples/openai/bslive.yml",
    {
        annotation: {
            type: cli({
                args: ["-i", "examples/openai/bslive.yml"],
            }),
            description: "",
        },
    },
    () => {
        test("server sent events", async ({ page, bs }) => {
            await page.goto(bs.path("/"), { waitUntil: "networkidle" });
            await expect(page.locator("#output")).toContainText(
                '"" "Thsis" " is"',
                {
                    timeout: 10000,
                },
            );
            const html = await page.innerHTML("#output");
            expect(html).toMatchSnapshot();
        });
        test("server sent events - from file", async ({ page, bs }) => {
            await page.goto(bs.named("openai-file", "/"), {
                waitUntil: "networkidle",
            });
            await expect(page.locator("pre")).toMatchAriaSnapshot(
                `- code: "\\"\\" \\"This\\" \\" is\\" \\" a\\" \\" test\\" \\".\\""`,
            );

            await test.step("reloading on file change", async () => {
                await page.evaluate(installMockHandler);

                bs.touch("examples/openai/sse/01.txt");
                // Wait for the reloadPage call
                await page.waitForFunction(() => {
                    return window.__playwright?.calls?.length === 1;
                });
                // Verify the calls
                const calls = await page.evaluate(readCalls);
                expect(calls).toStrictEqual([
                    [
                        {
                            kind: "reloadPage",
                        },
                    ],
                ]);
            });
        });
    },
);
