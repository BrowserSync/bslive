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
                {
                    line: "watchers.before.before",
                    prefix: "kBgX3v.0.817cQQ",
                },
                {
                    line: "watchers.before.after",
                    prefix: "kBgX3v.1.rlqY1s",
                },
                {
                    line: "Start A",
                    prefix: "kBgX3v.2.FLTr2X",
                },
                {
                    line: "End A",
                    prefix: "kBgX3v.3.pnDswR",
                },
            ]);
        });
    },
);
