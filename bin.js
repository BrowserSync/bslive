#!/usr/bin/env node
const {startSync} = require("./index.js");
let r = startSync(process.argv.slice(1));
console.log(r);
