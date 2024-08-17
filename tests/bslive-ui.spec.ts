import {bstest, test} from './utils';

test.describe('Browsersync bslive 404 UI', {
  annotation: {
    type: bstest({
      input: 'examples/basic/headers.yml'
    })
  }
}, () => {

  test('shows the UI', async ({page, request, bs}) => {
    await page.goto(bs.path('/__bslive'));
    await page.locator('bs-header').waitFor({timeout: 1000});
  });
})
