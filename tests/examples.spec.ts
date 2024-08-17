import {bstest, test} from './utils';
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

test.describe('examples/basic/delays.yml', {
  annotation: {
    type: bstest({
      input: 'examples/basic/delays.yml'
    }),
    description: ''
  }
}, () => {
  test('first delay item', async ({request, bs}) => {
    const start = Date.now();
    const response = await request.get(bs.path('/'));

    const body = await response.body();
    const diff = Date.now() - start;

    expect(body.toString()).toBe(`first - 200ms delay`)
    expect(diff).toBeGreaterThan(200)
    expect(diff).toBeLessThan(300)
  });
  test('500ms delay', async ({request, bs}) => {
    const start = Date.now();
    const response = await request.get(bs.path('/500'));

    const body = await response.body();
    const diff = Date.now() - start;

    expect(body.toString()).toBe(`second - 500ms delay`)
    expect(diff).toBeGreaterThan(500)
    expect(diff).toBeLessThan(600)
  });
})
