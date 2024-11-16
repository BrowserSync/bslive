import { bstest, test } from "./utils";
import { expect } from "@playwright/test";

test.describe("examples/basic/inject.yml", {
  annotation: {
    type: bstest({
      input: "examples/basic/inject.yml",
    }),
    description: "",
  },
}, () => {
  test("inject bslive:connector", async ({ request, bs }) => {
    const response = await request.get(bs.path("/"), {
      headers: {
        accept: "text/html",
      },
    });
    const body = await response.body();
    expect(body.toString()).toMatchSnapshot();

    {
      const response = await request.get(bs.path("/form.html"), {
        headers: {
          accept: "text/html",
        },
      });
      const body = await response.body();
      expect(body.toString()).toMatchSnapshot();
    }
  });
});
