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
                            task_id: "11547041707440023642",
                            line: "t1",
                            prefix: "[Yskbag]",
                        },
                    },
                },
                {
                    kind: "OutputLine",
                    payload: {
                        kind: "Stdout",
                        payload: {
                            task_id: "1771583503751589290",
                            line: "t2",
                            prefix: "[Fp4O58]",
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
            const lines = await run.waitForOutput("TaskTreeDisplay", 1);
            expect(lines).toStrictEqual([
                {
                    kind: "TaskTreeDisplay",
                    payload: {
                        tree: {
                            label: "[bM] Seq: 1 task(s)",
                            nodes: [
                                {
                                    label: "[mp6u7r] Seq: 2 task(s)",
                                    nodes: [
                                        {
                                            label: "[pW6OUa] − Runnable::Sh echo 'dry-t1'",
                                            nodes: [],
                                        },
                                        {
                                            label: "[K4AiCF] − Runnable::Sh echo 'dry-t2'",
                                            nodes: [],
                                        },
                                    ],
                                },
                            ],
                        },
                    },
                },
            ]);
        });
    },
);
