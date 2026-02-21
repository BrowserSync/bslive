import { cli, test } from "./utils";
import { expect } from "@playwright/test";

test.describe(
    "examples/watch/watch-fail.yml",
    {
        annotation: {
            type: cli({
                args: "-i examples/watch/watch-fail.yml".split(" "),
            }),
            description: "",
        },
    },
    () => {
        test("ignoring failures in a task sequence", async ({
            page,
            bs,
            request,
        }) => {
            await page.goto(bs.named("watch-ignore-failures", "/"), {
                waitUntil: "networkidle",
            });
            bs.touch("examples/watch/src/index.html");
            const lines = await bs.outputLines(2);
            expect(lines).toStrictEqual([
                {
                    line: "start.a",
                    prefix: "[KWpxOO]",
                    task_id: "12811876391347057540",
                },
                {
                    line: "start.b",
                    prefix: "[aP4ZWs]",
                    task_id: "9235661974223744739",
                },
            ]);
        });
    },
);
