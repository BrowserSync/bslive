import { cli, test } from "./utils";
import { expect } from "@playwright/test";

test.describe(
    "examples/watch/watch.yml",
    {
        annotation: {
            type: cli({
                args: "-i examples/watch/watch.yml".split(" "),
            }),
            description: "",
        },
    },
    () => {
        test("default watcher", async ({ page, bs, request }) => {
            await page.goto(bs.path("/"), { waitUntil: "networkidle" });
            bs.touch("examples/watch/src/index.html");
            expect(await bs.didOutput("FilesChanged")).toBe(true);
        });
    },
);
test.describe(
    "examples/watch/watch-ignored.yml",
    {
        annotation: {
            type: cli({
                args: "-i examples/watch/watch-ignored.yml".split(" "),
            }),
            description: "",
        },
    },
    () => {
        test("using the ignore option", async ({ page, bs, request }) => {
            await page.goto(bs.path("/"), { waitUntil: "networkidle" });
            bs.touch("examples/watch/src/index.html");
            await page.waitForTimeout(50);
            const messages = bs.messages.filter(
                (x) => x.kind === "FilesChanged",
            );
            expect(messages).toHaveLength(0);
        });
    },
);

test.describe(
    "examples/watch/watch-runner.yml",
    {
        annotation: {
            type: cli({
                args: "-i examples/watch/watch-runner.yml".split(" "),
            }),
            description: "",
        },
    },
    () => {
        test("overriding `run`", async ({ page, bs, request }) => {
            await page.goto(bs.path("/"), { waitUntil: "networkidle" });

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

            bs.touch("examples/watch/src/style.css");

            await requestPromise;

            // here we make sure that the regular 'external' event is not show,
            // because it's overridden in `watch-runner.yml`
            const messages = bs.messages.filter(
                (x) => x.kind === "FilesChanged",
            );
            expect(messages).toHaveLength(0);
        });
    },
);
