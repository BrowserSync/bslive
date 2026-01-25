import { cli, test } from "./utils";
import { expect } from "@playwright/test";

test.describe(
    "examples/watch/watch-graceful.yml",
    {
        annotation: {
            type: cli({
                args: "-i examples/watch/watch-graceful.yml".split(" "),
            }),
            description: "A task should be allowed to exit gracefully after timeout",
        },
    },
    () => {
        test("graceful exit on timeout", async ({ page, bs }) => {
            await page.goto(bs.named("graceful-exit", "/"), {
                waitUntil: "networkidle",
            });

            bs.touch("examples/watch/src/index.html");

            // We expect the task to start, then timeout after 1s, then cleanup for 2s.
            // Total time expected ~3-4s.
            
            // Wait for enough time
            await new Promise(resolve => setTimeout(resolve, 5000));

            const allLines = (await bs.outputLines(10)).map(l => l.line);
            
            console.log('allLines', allLines);

            expect(allLines.some(l => l.includes("Received SIGTERM"))).toBe(true);
            expect(allLines.some(l => l.includes("Cleanup complete"))).toBe(true);
        });
    },
);
