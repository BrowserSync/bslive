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
            await page.goto(bs.path("/"), { waitUntil: "networkidle" });
            expect(text).toContain("Hello from playground.md");
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
