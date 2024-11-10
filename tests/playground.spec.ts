import {bstest, test} from './utils';
import {expect} from "@playwright/test";

test.describe('examples/markdown/playground.md', {
  annotation: {
    type: bstest({
      input: 'examples/markdown/playground.md'
    }),
    description: ''
  }
}, () => {
  test('playground', async ({page, bs}) => {
    const text: string[] = [];
    page.on('console', (msg) => text.push(msg.text()));
    await page.goto(bs.path('/'), {waitUntil: 'networkidle'})
    expect(text).toContain('Hello from playground.md')
  });
})