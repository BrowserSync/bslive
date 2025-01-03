import { cli, test } from "./utils";
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
    },
);
