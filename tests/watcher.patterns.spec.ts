import { cli, test } from "./utils";
import { expect } from "@playwright/test";

test.describe(
    "examples/watch/watch-patterns.yml",
    {
        annotation: {
            type: cli({
                args: "-i examples/watch/watch-patterns.yml".split(" "),
            }),
            description: "",
        },
    },
    () => {
        test("patterns in globs", async ({ page, bs }) => {
            await page.goto(bs.named("watch-patterns", "/"), {
                waitUntil: "networkidle",
            });

            bs.touch("examples/watch/src/index.html");
            bs.touch("examples/watch/src/style.css");

            await page.waitForTimeout(50);

            let m = await bs.waitForOutput("OutputLine");

            expect(m).toStrictEqual([
                {
                    kind: "OutputLine",
                    payload: {
                        kind: "Stdout",
                        payload: {
                            line: "something inside examples/watch/src/ changed",
                            prefix: "[kW5WSJ]",
                            task_id: "12843725078283625749",
                        },
                    },
                },
            ]);
        });
    },
);
