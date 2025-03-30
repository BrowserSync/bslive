import { cli, test } from "./utils";
import { expect } from "@playwright/test";

test.describe(
    "examples/watcher/npm.yml",
    {
        annotation: {
            type: cli({
                args: "-i examples/watcher/npm.yml".split(" "),
            }),
            description: "",
        },
    },
    () => {
        test("ignoring at the root", async ({ page, bs, request }) => {
            // Array to store console messages
            const messages: [type: string, text: string][] = [];

            // Listen to console messages on the page
            page.on("console", (msg) => {
                messages.push([msg.type(), msg.text()]);
            });

            // Navigate to the page and wait until network is idle
            await page.goto(bs.path("/"), { waitUntil: "networkidle" });

            await expect(page.getByRole("list")).toContainText("a");
            await expect(page.getByRole("list")).toContainText("b");
        });
    },
);
