import { test as base } from "@playwright/test";
import { execSync, fork } from "node:child_process";
import { join } from "node:path";
import * as z from "zod";
import {
    externalEventsSchema,
    getServersMessageResponseSchema,
    internalEventsDTOSchema,
} from "../generated/schema.mjs";
import { existsSync } from "node:fs";

const either = z.union([internalEventsDTOSchema, externalEventsSchema]);

declare global {
    interface Window {
        __playwright?: {
            calls?: any[];
            record?: (...args: any[]) => void;
        };
    }
}

const messageSchema = z.discriminatedUnion("kind", [
    z.object({
        kind: z.literal("ready"),
        urls: z.object({
            local: z.string(),
            ui: z.string(),
        }),
        cwd: z.string(),
    }),
]);
type Msg = z.infer<typeof messageSchema>;
const inputSchema = z.object({
    input: z.string(),
});
const cliInputSchema = z.object({
    args: z.array(z.string()),
});

export function cli(input: z.infer<typeof cliInputSchema>) {
    return JSON.stringify(input, null, 2);
}

interface NextArgs {
    stdout: { lines: { count: number; after: number } };
}

type TServersResp = z.infer<typeof getServersMessageResponseSchema>;

export const test = base.extend<{
    bs: {
        url: string;
        cwd: string;
        data: TServersResp;
        servers: { url: string }[];
        child: any;
        path: (path: string) => string;
        named: (name: string, path: string) => string;
        stdout: string[];
        touch: (path: string) => void;
        api: (kind: "events") => string;
        // next: (args: NextArgs) => Promise<string[]>;
    };
}>({
    bs: async ({}, use, testInfo) => {
        const json = JSON.parse(testInfo.annotations[0].type);
        let ann;
        if ("args" in json) {
            ann = cliInputSchema.parse(json);
        } else {
            throw new Error("unreachable?");
        }

        const test_dir = ["tests"];
        const cwd = process.cwd();
        const base = join(cwd, ...test_dir);
        const file = join(base, "..", "bin.js");
        const stdout: string[] = [];

        let child = fork(file, ["-f", "json", ...ann.args], {
            cwd,
            stdio: "pipe",
        });

        const lines: string[] = [];
        const servers_changed_msg: Promise<TServersResp> = new Promise(
            (res, rej) => {
                const handler = (chunk: Buffer) => {
                    for (let line of chunk.toString().split("\n")) {
                        if (line.trim() === "") continue;
                        lines.push(line);
                        console.log("--", line);
                        const json = JSON.parse(line);
                        const parsed = either.safeParse(json);
                        if (parsed.error) {
                            // console.log(parsed.error.message)
                            // rej(parsed.error)
                        } else {
                            if (parsed.data.kind === "ServersChanged") {
                                res(parsed.data.payload as TServersResp);
                                child.stdout?.off("data", handler);
                            }
                        }
                    }
                };
                child.stdout?.on("data", handler);
            },
        );
        child.stderr?.on("data", (d) => console.error(d.toString()));
        const closed = new Promise((res, rej) => {
            child.on("disconnect", (...args) => {
                console.log("did disconnect", ...args);
            });
            child.on("close", (err, signal) => {
                if (err) {
                    if (err !== 0) {
                        console.log("did close with error code", err);
                        console.log(lines);
                        return rej(err);
                    }
                }
                console.log("did close cleanly", { signal });
                res(signal);
            });
            child.on("exit", (err, signal) => {
                console.log("did exit", { err, signal });
            });
            child.on("error", (err) => {
                console.error("did error", err);
            });
        });
        const data = await servers_changed_msg;
        const servers = data.servers.map((s) => {
            return { url: "http://" + s.socket_addr, identity: s.identity };
        });

        await use({
            url: "msg.urls.local",
            cwd: "msg.cwd",
            child,
            data,
            servers,
            path(path: string) {
                const url = new URL(path, servers[0].url);
                return url.toString();
            },
            named(server_name: string, path: string) {
                const server = servers.find((x) => {
                    if (x.identity.kind === "Named") {
                        return x.identity.payload.name === server_name;
                    }
                    return false;
                });
                if (!server) {
                    throw new Error(
                        "server not found with name: " + server_name,
                    );
                }
                const url = new URL(path, server.url);
                return url.toString();
            },
            api(kind: "events") {
                switch (kind) {
                    case "events":
                        return this.path("/__bs_api/events");
                }
                throw new Error("unreachable");
            },
            stdout,
            touch: (path: string) => {
                touchFile(join(cwd, path));
            },
        });

        child.kill("SIGTERM");

        await closed;
    },
});

function touchFile(filePath: string) {
    execSync(`touch ${filePath}`);
}

export function installMockHandler() {
    window.__playwright = {
        calls: [],
        record: (...args) => {
            window.__playwright?.calls?.push(args);
        },
    };
}

export function readCalls() {
    return window.__playwright?.calls;
}
