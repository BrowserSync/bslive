import test from 'ava'

import { start } from '../index.js'

test('sum from native', (t) => {
  t.is(typeof start, "function")
})
