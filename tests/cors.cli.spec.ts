import { cli, test } from "./utils";
import { expect } from "@playwright/test";

test.describe(
    "without cors cli args",
    {
        annotation: {
            type: cli({
                args: ["examples/basic/public"],
            }),
            description: "",
        },
    },
    () => {
        test("not adding cors headers (control)", async ({ request, bs }) => {
            const response = await request.get(bs.path("/"));
            const headers = Object.keys(response.headers()).sort();
            expect(headers).toEqual([
                "accept-ranges",
                "cache-control",
                "content-length",
                "content-type",
                "date",
                "expires",
                "last-modified",
                "pragma",
            ]);
        });
    },
);
test.describe(
    "with cors cli args",
    {
        annotation: {
            type: cli({
                args: ["start", "examples/basic/public", "--cors"],
            }),
            description: "",
        },
    },
    () => {
        test("adding cors headers", async ({ request, bs }) => {
            const response = await request.get(bs.path("/"));
            const headers = response.headers();
            const keys = Object.keys(headers).sort();
            const pairs = [];
            for (let key of keys) {
                if (key === "date") {
                    pairs.push([key, "--datetime--"]);
                } else if (key === "last-modified") {
                    pairs.push([key, "--datetime--"]);
                } else {
                    pairs.push([key, headers[key]]);
                }
            }
            expect(pairs).toEqual([
                ["accept-ranges", "bytes"],
                ["access-control-allow-origin", "*"],
                ["access-control-expose-headers", "*"],
                ["cache-control", "no-store, no-cache, must-revalidate"],
                ["content-length", "445"],
                ["content-type", "text/html"],
                ["date", "--datetime--"],
                ["expires", "0"],
                ["last-modified", "--datetime--"],
                ["pragma", "no-cache"],
                [
                    "vary",
                    "origin, access-control-request-method, access-control-request-headers",
                ],
            ]);
        });
    },
);
