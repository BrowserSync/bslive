#!/usr/bin/env node
console.log('from cli package');

const { startSync } = require("./index.js");
let r = startSync(process.argv.slice(1));
console.log(r);
