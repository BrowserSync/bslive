import {bstest, test} from "./utils";
import {expect} from "@playwright/test";

test.describe('examples/basic/headers.yml', {
  annotation: {
    type: bstest({
      input: 'examples/basic/headers.yml'
    }),
    description: ''
  }
}, () => {
  test('first item /', async ({request, bs}) => {
    // Send a GET request to the base URL
    const response = await request.get(bs.path('/'));

    // Extract headers from the response
    const headers = response.headers();

    // Extract body from the response
    const body = await response.body();

    // Assert that the content-type header is 'application/json'
    expect(headers['content-type']).toBe('application/json')

    // Assert that the body content is '[ 1, 2 ]'
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
