import {bstest, test} from './utils';
import {expect} from "@playwright/test";

test.describe('examples/basic', {
  annotation: {
    type: bstest({
      input: 'examples/basic/headers.yml'
    }),
    description: ''
  }
}, () => {
  test('first item /', async ({request, bs}) => {
    const response = await request.get(bs.path('/'));
    const headers = response.headers();
    const body = await response.body();
    expect(headers['content-type']).toBe('application/json')
    expect(body.toString()).toBe(`[ 1, 2 ]`)
  });
  test('/other', async ({request, bs}) => {
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
