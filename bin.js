#!/usr/bin/env node
const { startBlocking } = require("./index.js");
let code = startBlocking(process.argv.slice(1));
process.exit(code);
