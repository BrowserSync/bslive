import {bstest, test} from './utils';
import {expect} from "@playwright/test";

test.describe('examples/basic', {
  annotation: {
    type: bstest({
      path: 'examples/basic/headers.yml'
    }),
    description: ''
  }
}, () => {
  test('headers', async ({request, bs}) => {
    const response = await request.get(bs.path('/other'));
    const headers = response.headers();
    const expected = {
      'vary': 'origin, access-control-request-method, access-control-request-headers',
      'access-control-allow-origin': 'localhost',
      'access-control-expose-headers': '*',
      abc: 'def',
    }
    expect(headers).toMatchObject(expected)
  });
})
