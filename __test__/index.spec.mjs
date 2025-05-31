import test from "ava";

import { start, startBlocking } from "../index.js";

test("start + startBlocking from native", (t) => {
    t.is(typeof startBlocking, "function");
    t.is(typeof start, "function");
});
