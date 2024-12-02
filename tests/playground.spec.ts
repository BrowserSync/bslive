import { bstest, test } from "./utils";
import { expect } from "@playwright/test";

test.describe(
    "examples/markdown/playground.md",
    {
        annotation: {
            type: bstest({
                input: "examples/markdown/playground.md",
            }),
            description: "",
        },
    },
    () => {
        test("markdown playground", async ({ page, bs }) => {
            const text: string[] = [];
            page.on("console", (msg) => text.push(msg.text()));

            /**
             * This first request should be picked up by the playground
             */
            await page.goto(bs.path("/"), { waitUntil: "networkidle" });
            const html = await page.locator(":root").innerHTML();
            expect(html).toMatchSnapshot();

            await test.step("serving dir alongside playground", async () => {
                /**
                 * This index.html request should bypass the playground and serve from disk
                 */
                await page.goto(bs.path("/index.html"), {
                    waitUntil: "networkidle",
                });
                await page.getByText("Edit me! - a full HTML").waitFor();
            });
        });
    },
);

test.describe(
    "examples/html/playground.html",
    {
        annotation: {
            type: bstest({
                input: "examples/html/playground.html",
            }),
            description: "",
        },
    },
    () => {
        test("html playground", async ({ page, bs }) => {
            const text: string[] = [];
            page.on("console", (msg) => text.push(msg.text()));
            await page.goto(bs.path("/"), { waitUntil: "networkidle" });
            await expect(page.locator("abc-element")).toContainText(
                "Hello World!",
            );
        });
    },
);
