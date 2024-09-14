import {bstest, test} from "./utils";
import {expect} from "@playwright/test";

test.describe('examples/basic/client.yml', {
  annotation: {
    type: bstest({
      input: 'examples/basic/client.yml'
    }),
    description: ''
  }
}, () => {
  test('configures log level', async ({request, bs}) => {
  });
})
