#!/usr/bin/env node
const {startSync} = require("./index.js");
let code = startSync(process.argv.slice(1));
process.exit(code);
