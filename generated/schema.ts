// Generated by ts-to-zod
import { z } from "zod";
import { type ChangeDTO, LogLevelDTO, ChangeKind, EventLevel } from "./dto";

export const logLevelDTOSchema = z.nativeEnum(LogLevelDTO);

export const clientConfigDTOSchema = z.object({
    log_level: logLevelDTOSchema,
});

export const connectInfoSchema = z.object({
    ws_path: z.string(),
    host: z.string().optional(),
});

export const debounceDTOSchema = z.object({
    kind: z.string(),
    ms: z.string(),
});

export const fileChangedDTOSchema = z.object({
    path: z.string(),
});

export const filesChangedDTOSchema = z.object({
    paths: z.array(z.string()),
});

export const serverIdentityDTOSchema = z.discriminatedUnion("kind", [
    z.object({
        kind: z.literal("Both"),
        payload: z.object({
            name: z.string(),
            bind_address: z.string(),
        }),
    }),
    z.object({
        kind: z.literal("Address"),
        payload: z.object({
            bind_address: z.string(),
        }),
    }),
    z.object({
        kind: z.literal("Named"),
        payload: z.object({
            name: z.string(),
        }),
    }),
    z.object({
        kind: z.literal("Port"),
        payload: z.object({
            port: z.number(),
        }),
    }),
    z.object({
        kind: z.literal("PortNamed"),
        payload: z.object({
            port: z.number(),
            name: z.string(),
        }),
    }),
]);

export const serverDTOSchema = z.object({
    id: z.string(),
    identity: serverIdentityDTOSchema,
    socket_addr: z.string(),
});

export const getActiveServersResponseDTOSchema = z.object({
    servers: z.array(serverDTOSchema),
});

export const injectConfigSchema = z.object({
    connect: connectInfoSchema,
    ctx_message: z.string(),
});

export const inputAcceptedDTOSchema = z.object({
    path: z.string(),
});

export const sseDTOOptsSchema = z.object({
    body: z.string(),
});

export const routeKindDTOSchema = z.discriminatedUnion("kind", [
    z.object({
        kind: z.literal("Html"),
        payload: z.object({
            html: z.string(),
        }),
    }),
    z.object({
        kind: z.literal("Json"),
        payload: z.object({
            json_str: z.string(),
        }),
    }),
    z.object({
        kind: z.literal("Raw"),
        payload: z.object({
            raw: z.string(),
        }),
    }),
    z.object({
        kind: z.literal("Sse"),
        payload: z.object({
            sse: sseDTOOptsSchema,
        }),
    }),
    z.object({
        kind: z.literal("Proxy"),
        payload: z.object({
            proxy: z.string(),
        }),
    }),
    z.object({
        kind: z.literal("Dir"),
        payload: z.object({
            dir: z.string(),
            base: z.string().optional(),
        }),
    }),
]);

export const serverChangeSchema = z.discriminatedUnion("kind", [
    z.object({
        kind: z.literal("Stopped"),
        payload: z.object({
            bind_address: z.string(),
        }),
    }),
    z.object({
        kind: z.literal("Started"),
        payload: z.undefined().optional(),
    }),
    z.object({
        kind: z.literal("Patched"),
        payload: z.undefined().optional(),
    }),
    z.object({
        kind: z.literal("Errored"),
        payload: z.object({
            error: z.string(),
        }),
    }),
]);

export const serverChangeSetItemSchema = z.object({
    identity: serverIdentityDTOSchema,
    change: serverChangeSchema,
});

export const serverChangeSetSchema = z.object({
    items: z.array(serverChangeSetItemSchema),
});

export const routeDTOSchema = z.object({
    path: z.string(),
    kind: routeKindDTOSchema,
});

export const serversChangedDTOSchema = z.object({
    servers_resp: getActiveServersResponseDTOSchema,
});

export const stderrLineDTOSchema = z.object({
    line: z.string(),
    prefix: z.string().optional(),
});

export const stdoutLineDTOSchema = z.object({
    line: z.string(),
    prefix: z.string().optional(),
});

export const stoppedWatchingDTOSchema = z.object({
    paths: z.array(z.string()),
});

export const watchingDTOSchema = z.object({
    paths: z.array(z.string()),
    debounce: debounceDTOSchema,
});

export const changeKindSchema = z.nativeEnum(ChangeKind);

export const changeDTOSchema: z.ZodSchema<ChangeDTO> = z.lazy(() =>
    z.discriminatedUnion("kind", [
        z.object({
            kind: z.literal("Fs"),
            payload: z.object({
                path: z.string(),
                change_kind: changeKindSchema,
            }),
        }),
        z.object({
            kind: z.literal("FsMany"),
            payload: z.array(changeDTOSchema),
        }),
    ]),
);

export const clientEventSchema = z.discriminatedUnion("kind", [
    z.object({
        kind: z.literal("Change"),
        payload: changeDTOSchema,
    }),
    z.object({
        kind: z.literal("WsConnection"),
        payload: clientConfigDTOSchema,
    }),
    z.object({
        kind: z.literal("Config"),
        payload: clientConfigDTOSchema,
    }),
]);

export const eventLevelSchema = z.nativeEnum(EventLevel);

export const outputLineDTOSchema = z.discriminatedUnion("kind", [
    z.object({
        kind: z.literal("Stdout"),
        payload: stdoutLineDTOSchema,
    }),
    z.object({
        kind: z.literal("Stderr"),
        payload: stderrLineDTOSchema,
    }),
]);

export const inputErrorDTOSchema = z.discriminatedUnion("kind", [
    z.object({
        kind: z.literal("MissingInputs"),
        payload: z.string(),
    }),
    z.object({
        kind: z.literal("InvalidInput"),
        payload: z.string(),
    }),
    z.object({
        kind: z.literal("NotFound"),
        payload: z.string(),
    }),
    z.object({
        kind: z.literal("InputWriteError"),
        payload: z.string(),
    }),
    z.object({
        kind: z.literal("PathError"),
        payload: z.string(),
    }),
    z.object({
        kind: z.literal("PortError"),
        payload: z.string(),
    }),
    z.object({
        kind: z.literal("DirError"),
        payload: z.string(),
    }),
    z.object({
        kind: z.literal("YamlError"),
        payload: z.string(),
    }),
    z.object({
        kind: z.literal("MarkdownError"),
        payload: z.string(),
    }),
    z.object({
        kind: z.literal("HtmlError"),
        payload: z.string(),
    }),
    z.object({
        kind: z.literal("Io"),
        payload: z.string(),
    }),
    z.object({
        kind: z.literal("UnsupportedExtension"),
        payload: z.string(),
    }),
    z.object({
        kind: z.literal("MissingExtension"),
        payload: z.string(),
    }),
    z.object({
        kind: z.literal("EmptyInput"),
        payload: z.string(),
    }),
    z.object({
        kind: z.literal("BsLiveRules"),
        payload: z.string(),
    }),
]);

export const internalEventsDTOSchema = z.discriminatedUnion("kind", [
    z.object({
        kind: z.literal("ServersChanged"),
        payload: getActiveServersResponseDTOSchema,
    }),
    z.object({
        kind: z.literal("TaskReport"),
        payload: z.object({
            id: z.string(),
        }),
    }),
]);

export const startupEventDTOSchema = z.discriminatedUnion("kind", [
    z.object({
        kind: z.literal("Started"),
        payload: z.undefined().optional(),
    }),
    z.object({
        kind: z.literal("FailedStartup"),
        payload: z.string(),
    }),
]);

export const serverDescSchema = z.object({
    routes: z.array(routeDTOSchema),
    id: z.string(),
});

export const externalEventsDTOSchema = z.discriminatedUnion("kind", [
    z.object({
        kind: z.literal("ServersChanged"),
        payload: serversChangedDTOSchema,
    }),
    z.object({
        kind: z.literal("Watching"),
        payload: watchingDTOSchema,
    }),
    z.object({
        kind: z.literal("WatchingStopped"),
        payload: stoppedWatchingDTOSchema,
    }),
    z.object({
        kind: z.literal("FileChanged"),
        payload: fileChangedDTOSchema,
    }),
    z.object({
        kind: z.literal("FilesChanged"),
        payload: filesChangedDTOSchema,
    }),
    z.object({
        kind: z.literal("InputFileChanged"),
        payload: fileChangedDTOSchema,
    }),
    z.object({
        kind: z.literal("InputAccepted"),
        payload: inputAcceptedDTOSchema,
    }),
    z.object({
        kind: z.literal("OutputLine"),
        payload: outputLineDTOSchema,
    }),
]);
