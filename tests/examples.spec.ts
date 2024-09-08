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


test.describe('examples/basic/inject.yml', {
  annotation: {
    type: bstest({
      input: 'examples/basic/inject.yml'
    }),
    description: ''
  }
}, () => {
  test('inject bslive:connector', async ({request, bs}) => {
    const response = await request.get(bs.path('/'), {
      headers: {
        accept: 'text/html'
      }
    });
    const body = await response.body();
    expect(body.toString()).toMatchSnapshot();

    {
      const response = await request.get(bs.path('/form.html'), {
        headers: {
          accept: 'text/html'
        }
      });
      const body = await response.body();
      expect(body.toString()).toMatchSnapshot();
    }
  });
})

test.describe('examples/basic/live-reload.yml', {
  annotation: {
    type: bstest({
      input: 'examples/basic/live-reload.yml'
    }),
    description: ''
  }
}, () => {
  test('live-reloading css', async ({page, bs}) => {
    await page.goto(bs.path('/'), {waitUntil: 'networkidle'})
    const requestPromise = page.waitForRequest((req) => {
      const url = new URL(req.url());
      return url.searchParams.has('livereload')
        && url.pathname === "/styles.css"
    }, {timeout: 2000});
    bs.touch('examples/basic/public/styles.css')
    await requestPromise;
  });
  test('reloads with HTML change', async ({page, bs, request}) => {
    page.on('console', (evt) => {
      console.log("PAGE LOG: ", evt.type(), evt.text())
    })
    await page.goto(bs.path('/'), {waitUntil: 'networkidle'})
    const change = {
      "kind": "Change",
      "payload": {
        "kind": "Fs",
        "payload": {
          "path": "index.html",
          "change_kind": "Changed"
        }
      }
    };
    await page.evaluate(() => {
      window.__playwright = {
        calls: [],
        record: (...args) => {
          window.__playwright?.calls?.push(args)
        }
      }
    });
    await request.post(bs.api('events'), {data: change});
    await page.waitForFunction(() => {
      return window.__playwright?.calls?.length === 1
    })
    const calls = await page.evaluate(() => {
      return window.__playwright?.calls
    })
    expect(JSON.stringify(calls, null, 2)).toMatchSnapshot();
  });
})

test.describe('examples/react-router/bslive.yaml', {
  annotation: {
    type: bstest({
      input: 'examples/react-router/bslive.yaml'
    }),
    description: ''
  }
}, () => {
  test('support client-side routing', async ({page, bs}) => {
    await page.goto(bs.path('/'), {waitUntil: 'networkidle'})
    await expect(page.locator('#root')).toContainText('API response from /abc[1,2,3]');
  });
  test('supports compressed responses', async ({page, bs}) => {
    const load = page.goto(bs.named('react-router-with-compression', '/'), {waitUntil: 'networkidle'})
    const requestPromise = page.waitForResponse((req) => {
      const url = new URL(req.url());
      return url.pathname.includes('assets/index')
        && url.pathname.endsWith('.js')
    }, {timeout: 2000});
    const [_, jsfile] = await Promise.all([load, requestPromise]);
    expect(jsfile?.headers()['content-encoding']).toBe('zstd')
  });
  test('does not compress by default', async ({page, bs}) => {
    const load = page.goto(bs.named('react-router', '/'), {waitUntil: 'networkidle'})
    const requestPromise = page.waitForResponse((req) => {
      const url = new URL(req.url());
      return url.pathname.includes('assets/index')
        && url.pathname.endsWith('.js')
    }, {timeout: 2000});
    const [_, jsfile] = await Promise.all([load, requestPromise]);
    expect(jsfile?.headers()['content-encoding']).toBeUndefined()
  });
})

declare global {
  interface Window {
    __playwright?: {
      calls?: any[],
      record?: (...args: any[]) => void
    }
  }
}
