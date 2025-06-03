import { cli, test } from "./utils";
import { expect } from "@playwright/test";

test.describe(
    "examples/watch/watch-before.yml",
    {
        annotation: {
            type: cli({
                args: "-i examples/watch/watch-before.yml".split(" "),
            }),
            description: "",
        },
    },
    () => {
        test("running before", async ({ page, bs, request }) => {
            await page.goto(bs.named("watch-before-tasks", "/"), {
                waitUntil: "networkidle",
            });

            const lines = await bs.outputLines(4);
            expect(lines).toStrictEqual([
                { line: "watchers.before.before", prefix: "[run]" },
                { line: "watchers.before.after", prefix: "[run]" },
                { line: "Start A", prefix: "[run]" },
                { line: "End A", prefix: "[run]" },
            ]);
        });
    },
);
