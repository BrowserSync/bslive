import test from "ava";

import { startBlocking, BsSystem } from "../index.js";

test("start + startBlocking from native", (t) => {
    t.is(typeof startBlocking, "function");
    t.deepEqual(typeof BsSystem, "function");
});
