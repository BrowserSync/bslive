import { cli, test } from "./utils";
import { expect } from "@playwright/test";

test.describe(
    "examples/markdown/playground.md",
    {
        annotation: {
            type: cli({
                args: ["-i", "examples/markdown/playground.md"],
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
            type: cli({
                args: ["-i", "examples/html/playground.html"],
            }),
            description: "",
        },
    },
    () => {
        test("html playground", async ({ page, bs }) => {
            const text: string[] = [];
            page.on("console", (msg) => text.push(msg.text()));
            let waitForImage = page.waitForResponse((res) => {
                const url = res.url();
                return url.includes("bg-01.jpg");
            });
            await page.goto(bs.path("/"), { waitUntil: "networkidle" });
            await expect(page.locator("abc-element")).toContainText(
                "Hello World!",
            );
            let imageResponse = await waitForImage;
            expect(imageResponse.ok()).toBeTruthy();
        });
    },
);

test.describe(
    "examples/html/js-playground.js",
    {
        annotation: {
            type: cli({
                args: ["-i", "examples/html/js-playground.js"],
            }),
            description: "",
        },
    },
    () => {
        test("js playground", async ({ page, bs }) => {
            await page.goto(bs.path("/"), { waitUntil: "networkidle" });
            await expect(
                page.getByText("did load from js-playground"),
            ).toBeVisible();
        });
    },
);
