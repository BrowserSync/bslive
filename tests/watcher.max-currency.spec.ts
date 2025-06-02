import { cli, test } from "./utils";
import { expect } from "@playwright/test";

test.describe(
    "examples/watch/watch-max-concurrency.yml",
    {
        annotation: {
            type: cli({
                args: "-i examples/watch/watch-max-concurrency.yml".split(" "),
            }),
            description: "",
        },
    },
    () => {
        test("max concurrency default (5)", async ({ page, bs, request }) => {
            await page.goto(bs.named("max-concurrency-2", "/"), {
                waitUntil: "networkidle",
            });

            bs.touch("examples/watch/src/index.html");
            const lines = await bs.outputLines(10);
            const starts = lines
                .slice(0, 3)
                .every((x) => x.line.startsWith("Start"));

            expect(starts).toBe(true);

            const ends = lines
                .slice(3, 6)
                .every((x) => x.line.startsWith("End"));

            expect(ends).toBe(true);
        });
    },
);
