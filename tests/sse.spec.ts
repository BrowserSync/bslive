import {bstest, test} from './utils';
import {expect} from "@playwright/test";

test.describe('examples/openai/bslive.yml', {
  annotation: {
    type: bstest({
      input: 'examples/openai/bslive.yml'
    }),
    description: ''
  }
}, () => {
  test('server sent events', async ({page, bs}) => {
    await page.goto(bs.path('/'), {waitUntil: 'networkidle'})
    const html = await page.innerHTML('#output');
    expect(html).toMatchSnapshot()
  });
})