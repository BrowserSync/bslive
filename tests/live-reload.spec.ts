import { bstest, installMockHandler, readCalls, test } from "./utils";
import { expect } from "@playwright/test";
import { z } from "zod";
import { clientEventSchema } from "../generated/schema";
import { ChangeKind } from "@browsersync/generated/dto";

test.describe(
    "examples/basic/live-reload.yml",
    {
        annotation: {
            type: bstest({
                input: "examples/basic/live-reload.yml",
            }),
            description: "",
        },
    },
    () => {
        test("live-reloading css", async ({ page, bs }) => {
            // Array to store console messages
            const messages: [type: string, text: string][] = [];

            // Listen to console messages on the page
            page.on("console", (msg) => {
                messages.push([msg.type(), msg.text()]);
            });

            // Navigate to the page and wait until network is idle
            await page.goto(bs.path("/"), { waitUntil: "networkidle" });

            // Set up the request waiting promise
            const requestPromise = page.waitForRequest(
                (req) => {
                    const url = new URL(req.url());
                    return (
                        url.searchParams.has("livereload") &&
                        url.pathname === "/styles.css"
                    );
                },
                { timeout: 2000 },
            );

            // Trigger the live-reload by touching the CSS file
            bs.touch("examples/basic/public/styles.css");
            await requestPromise;

            // Filter the log messages for specific content and assert
            const log = messages.filter(
                ([, text]) =>
                    text ===
                    "[debug] found 1 LINKed stylesheets, 1 @imported stylesheets",
            );
            expect(log).toHaveLength(1);
        });
        test("reloads with HTML change", async ({ page, bs, request }) => {
            await page.goto(bs.path("/"), { waitUntil: "networkidle" });

            // Define the change event
            const change: z.infer<typeof clientEventSchema> = {
                kind: "Change",
                payload: {
                    kind: "Fs",
                    payload: {
                        path: "index.html",
                        change_kind: ChangeKind.Changed,
                    },
                },
            };

            // Install mock handler and send change event
            await page.evaluate(installMockHandler);
            await request.post(bs.api("events"), { data: change });

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
        test("no css reloads with HTML + CSS change", async ({
            page,
            bs,
            request,
        }) => {
            // Navigate to the page and wait until network is idle
            await page.goto(bs.path("/"), { waitUntil: "networkidle" });

            // Define the change event
            const change: z.infer<typeof clientEventSchema> = {
                kind: "Change",
                payload: {
                    kind: "FsMany",
                    payload: [
                        {
                            kind: "Fs",
                            payload: {
                                path: "reset.css",
                                change_kind: ChangeKind.Changed,
                            },
                        },
                        {
                            kind: "Fs",
                            payload: {
                                path: "index.html",
                                change_kind: ChangeKind.Changed,
                            },
                        },
                    ],
                },
            };

            // Install mock handler and send change event
            await page.evaluate(installMockHandler);
            await request.post(bs.api("events"), { data: change });

            // Wait for a short period
            await page.waitForTimeout(500);

            // Verify the recorded calls
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
