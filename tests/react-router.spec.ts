import { bstest, test } from "./utils";
import { expect } from "@playwright/test";

test.describe(
  "examples/react-router/bslive.yaml",
  {
    annotation: {
      type: bstest({
        input: "examples/react-router/bslive.yaml",
      }),
      description: "",
    },
  },
  () => {
    test("support client-side routing", async ({ page, bs }) => {
      await page.goto(bs.path("/"), { waitUntil: "networkidle" });
      await expect(page.locator("#root")).toContainText(
        "API response from /abc[1,2,3]",
      );
    });
    test("supports compressed responses", async ({ page, bs }) => {
      // Navigate to the page and wait until the network becomes idle
      const load = page.goto(bs.named("react-router-with-compression", "/"), {
        waitUntil: "networkidle",
      });

      // Set up the request waiting promise
      const requestPromise = page.waitForResponse(
        (req) => {
          const url = new URL(req.url());
          return (
            url.pathname.includes("assets/index") &&
            url.pathname.endsWith(".js")
          );
        },
        { timeout: 2000 },
      );

      // Wait for both navigation and request to complete
      const [_, jsfile] = await Promise.all([load, requestPromise]);

      // Assert that the content-encoding header is 'zstd'
      expect(jsfile?.headers()["content-encoding"]).toBe("zstd");
    });
    test("does not compress by default", async ({ page, bs }) => {
      const load = page.goto(bs.named("react-router", "/"), {
        waitUntil: "networkidle",
      });

      const requestPromise = page.waitForResponse(
        (req) => {
          const url = new URL(req.url());
          return (
            url.pathname.includes("assets/index") &&
            url.pathname.endsWith(".js")
          );
        },
        { timeout: 2000 },
      );

      // Wait for both navigation and request to complete
      const [_, jsfile] = await Promise.all([load, requestPromise]);

      // Assert that the content-encoding header is undefined by default
      expect(jsfile?.headers()["content-encoding"]).toBeUndefined();
    });
  },
);
