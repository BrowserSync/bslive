import z from "zod";
import {fork} from "node:child_process";
import {internalEventsDTOSchema, externalEventsSchema} from "./crates/bsnext_client/generated/schema.js";

const m = fork('./bin.js', ['-f', 'json'], {stdio: 'pipe'});
const either = z.union([internalEventsDTOSchema, externalEventsSchema]);
const lines = []

/**
 * @param {import("zod").infer<either>} input
 */
function handle(input) {
    switch (input.kind) {
        case "ServersChanged":
            process.send(input);
    }
}

m.stdout.on('data', (chunk) => {
    for (let line of chunk.toString().split('\n')) {
        if (line.trim() === "") continue;
        lines.push(line);
        console.log(line);
        const json = JSON.parse(line);
        const parsed = either.safeParse(json);
        if (parsed.error) {
            // console.log(parsed.error)
        } else {
            handle(parsed.data)
        }
    }
})
m.stderr.on('data', (e) => {
    console.log("error", e.toString())
})
m.on('spawn', (...args) => {
    console.log('did spawn', ...args)
    setTimeout(() => {
        console.log('will kill')
        m.kill();
    }, 2000)
})
m.on('disconnect', (...args) => {
    console.log('did disconnect', ...args)
})
m.on('close', (err, signal) => {
    if (err) {
        if (err !== 0) {
            console.log('did close with error code', err)
            process.exit(err)
        }
    }
    console.log('did close', {err, signal})
})
m.on('exit', (err, signal) => {
    console.log('did exit', {err, signal})
})
m.on('error', (err) => {
    console.error('did error', err)
})
