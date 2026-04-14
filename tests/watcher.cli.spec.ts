import { cli, installMockHandler, readCalls, test } from "./utils";
import { expect } from "@playwright/test";

test.describe(
    "watch from cli",
    {
        annotation: {
            type: cli({
                args: [
                    "watch",
                    "examples/watch/src",
                    "--run",
                    "sh:echo did change",
                ],
            }),
            description: "",
        },
    },
    () => {
        test("watcher with output", async ({ page, bs, request }) => {
            bs.touch("examples/watch/src/index.html");
            let lines = await bs.waitForOutput("OutputLine", 1);
            expect(JSON.stringify(lines, null, 2)).toMatchSnapshot();
        });
    },
);

test.describe(
    "trailing args + watcher from cli [CONTROL]",
    {
        annotation: {
            type: cli({
                // args: ["examples/watch/src", "--watch.paths", "examples/watch/src/01.txt", "--watch.run", "sh: echo 01"]
                args: ["examples/watch/src"],
            }),
            description: "",
        },
    },
    () => {
        test("normal", async ({ page, bs, request }) => {
            let out = bs.waitForOutput("Watching");
            await page.goto(bs.path("/"));
            bs.touch("examples/watch/src/01.txt");
            let lines = await out;
            expect(JSON.stringify(lines, null, 2)).toMatchSnapshot();
        });
    },
);
test.describe(
    "trailing args + watcher from cli with watch override",
    {
        annotation: {
            type: cli({
                args: [
                    "examples/watch/src",
                    "--watch.paths",
                    "examples/watch/src/01.txt",
                    "--watch.run",
                    "sh: echo 01",
                    "--watch.run",
                    "bslive:notify-server",
                ],
            }),
            description: "",
        },
    },
    () => {
        test("path and command override", async ({ page, bs, request }) => {
            let out = bs.waitForOutput("Watching");
            await page.goto(bs.path("/"));

            // Install mock handler
            await page.evaluate(installMockHandler);

            // simulate the file change event
            bs.touch("examples/watch/src/01.txt");

            // Wait for the reloadPage call
            await page.waitForFunction(() => {
                return window.__playwright?.calls?.length === 1;
            });

            // Verify the calls
            const calls = await page.evaluate(readCalls);
            expect(calls).toStrictEqual([
                [
                    {
                        kind: "reloadPage",
                    },
                ],
            ]);
        });
    },
);

test.describe(
    "watch from cli with --initial",
    {
        annotation: {
            type: cli({
                args: [
                    "watch",
                    "examples/watch/src",
                    "--run",
                    "sh: echo did run initial",
                    "--initial",
                ],
            }),
            description: "",
        },
    },
    () => {
        test("runs commands once before watching", async ({
            page,
            bs,
            request,
        }) => {
            let lines = await bs.outputLines(1);
            expect(JSON.stringify(lines, null, 2)).toContain("did run initial");
        });
    },
);
