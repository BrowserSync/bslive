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
                    prefix: "[M3CpiT]",
                    task_id: "897071020278212152",
                },
                {
                    line: "watchers.before.after",
                    prefix: "[5V2TUX]",
                    task_id: "6224909781424175219",
                },
                {
                    line: "Start A",
                    prefix: "[AVCu6z]",
                    task_id: "4603670863854342654",
                },
                {
                    line: "End A",
                    prefix: "[xD5VGO]",
                    task_id: "1713922480753373637",
                },
            ]);
        });
    },
);
