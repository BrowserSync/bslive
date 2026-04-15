import { cli, test } from "./utils";
import { expect } from "@playwright/test";

test.describe(
    "run",
    {
        annotation: {
            type: cli({
                args: ["run", "--sh", "echo 't1'", "--sh", "echo 't2'"],
            }),
            description: "",
        },
    },
    () => {
        test("running 2 commands on cli", async ({ run }) => {
            // bs.stdout
            const lines = await run.waitForOutput("OutputLine", 2);
            expect(lines).toStrictEqual([
                {
                    kind: "OutputLine",
                    payload: {
                        kind: "Stdout",
                        payload: {
                            line: "t1",
                            prefix: "cHBgje.0.yzQkcw.0.G0rXKy",
                        },
                    },
                },
                {
                    kind: "OutputLine",
                    payload: {
                        kind: "Stdout",
                        payload: {
                            line: "t2",
                            prefix: "cHBgje.0.yzQkcw.1.izQ8ML",
                        },
                    },
                },
            ]);
        });
    },
);

test.describe(
    "dry run",
    {
        annotation: {
            type: cli({
                args: [
                    "run",
                    "--sh",
                    "echo 'dry-t1'",
                    "--sh",
                    "echo 'dry-t2'",
                    "--dry",
                ],
            }),
            description: "",
        },
    },
    () => {
        test("running 2 commands on cli", async ({ run }) => {
            // bs.stdout
            const lines = await run.waitForOutput("TaskTreePreview", 1);
            expect(lines).toStrictEqual([
                {
                    kind: "TaskTreePreview",
                    payload: {
                        tree: {
                            id: "voCKol",
                            label: "[voCKol] seq: 1",
                            nodes: [
                                {
                                    id: "voCKol.0.aPjCfW",
                                    label: "[voCKol.0.aPjCfW] seq: 2",
                                    nodes: [
                                        {
                                            id: "voCKol.0.aPjCfW.0.4uPDw3",
                                            label: "[voCKol.0.aPjCfW.0.4uPDw3] ShCmd echo 'dry-t1'",
                                            nodes: [],
                                        },
                                        {
                                            id: "voCKol.0.aPjCfW.1.K4LKHN",
                                            label: "[voCKol.0.aPjCfW.1.K4LKHN] ShCmd echo 'dry-t2'",
                                            nodes: [],
                                        },
                                    ],
                                },
                            ],
                        },
                        will_exec: false,
                    },
                },
            ]);
        });
    },
);
