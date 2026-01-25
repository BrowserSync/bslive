import { cli, test } from "./utils";
import { expect } from "@playwright/test";

test.describe(
    "examples/watch/watch-cancellation.yml",
    {
        annotation: {
            type: cli({
                args: "-i examples/watch/watch-cancellation.yml".split(" "),
            }),
            description: "A failing sibling task should cause others to be cancelled",
        },
    },
    () => {
        test("cancellation of sibling tasks", async ({ page, bs }) => {
            await page.goto(bs.named("watch-cancellation", "/"), {
                waitUntil: "networkidle",
            });

            bs.touch("examples/watch/src/index.html");

            // We expect "starting failing task" and "starting succeeding task" to appear
            // then Task 1 fails after 0.1s.
            // Task 2 should be cancelled and NOT produce "completed succeeding task"

            // Wait for the outputs. We expect at least 2 lines (the two starts)
            const lines = await bs.outputLines(2);

            expect(lines.some(l => l.line === "starting failing task")).toBe(true);
            expect(lines.some(l => l.line === "starting succeeding task")).toBe(true);

            // Wait for enough time for Task 1 to fail and Task 2 to potentially finish if NOT cancelled.
            // Task 1 fails after 0.1s.
            // Task 2 would finish after 1s.
            // If we wait 1.5s, and cancellation worked, "completed succeeding task" should NOT be there.
            await new Promise(resolve => setTimeout(resolve, 1500));

            const allLines = (await bs.outputLines(2)).map(l => l.line);
            
            expect(allLines).not.toContain("completed succeeding task");
        });
    },
);
