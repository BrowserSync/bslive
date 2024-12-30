import { cli, test } from "./utils";

test.describe(
    "examples/basic/client.yml",
    {
        annotation: {
            type: cli({
                args: ["-i", "examples/basic/client.yml"],
            }),
            description: "",
        },
    },
    () => {
        test("configures log level", async ({ request, bs }) => {});
    },
);
