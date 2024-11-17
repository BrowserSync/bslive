import { bstest, test } from "./utils";

test.describe(
    "examples/basic/client.yml",
    {
        annotation: {
            type: bstest({
                input: "examples/basic/client.yml",
            }),
            description: "",
        },
    },
    () => {
        test("configures log level", async ({ request, bs }) => {});
    },
);
