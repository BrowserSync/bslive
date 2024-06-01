// Generated by ts-to-zod
import { z } from "zod";
import { EventLevel, ChangeKind, ChangeDTO } from "./dto";

export const routeKindDTOSchema = z.union([
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
      sse: z.string(),
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
    }),
  }),
]);

export const routeDTOSchema = z.object({
  path: z.string(),
  kind: routeKindDTOSchema,
});

export const serverDescSchema = z.object({
  routes: z.array(routeDTOSchema),
  id: z.string(),
});

export const identityDTOSchema = z.union([
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
]);

export const serverDTOSchema = z.object({
  id: z.string(),
  identity: identityDTOSchema,
  socket_addr: z.string(),
});

export const getServersMessageResponseSchema = z.object({
  servers: z.array(serverDTOSchema),
});

export const serverChangeSchema = z.union([
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
]);

export const serverChangeSetItemSchema = z.object({
  identity: identityDTOSchema,
  change: serverChangeSchema,
});

export const serverChangeSetSchema = z.object({
  items: z.array(serverChangeSetItemSchema),
});

export const serversStartedSchema = z.object({
  servers_resp: getServersMessageResponseSchema,
  changeset: serverChangeSetSchema,
});

export const eventLevelSchema = z.nativeEnum(EventLevel);

export const stoppedWatchingSchema = z.object({
  paths: z.array(z.string()),
});

export const fileChangedSchema = z.object({
  path: z.string(),
});

export const filesChangedDTOSchema = z.object({
  paths: z.array(z.string()),
});

export const inputAcceptedSchema = z.object({
  path: z.string(),
});

export const inputErrorDTOSchema = z.union([
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
]);

export const debounceDTOSchema = z.object({
  kind: z.string(),
  ms: z.string(),
});

export const changeKindSchema = z.nativeEnum(ChangeKind);

export const watchingSchema = z.object({
  paths: z.array(z.string()),
  debounce: debounceDTOSchema,
});

export const changeDTOSchema: z.ZodSchema<ChangeDTO> = z.lazy(() =>
  z.union([
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

export const externalEventsSchema = z.union([
  z.object({
    kind: z.literal("ServersStarted"),
    payload: serversStartedSchema,
  }),
  z.object({
    kind: z.literal("Watching"),
    payload: watchingSchema,
  }),
  z.object({
    kind: z.literal("WatchingStopped"),
    payload: stoppedWatchingSchema,
  }),
  z.object({
    kind: z.literal("FileChanged"),
    payload: fileChangedSchema,
  }),
  z.object({
    kind: z.literal("FilesChanged"),
    payload: filesChangedDTOSchema,
  }),
  z.object({
    kind: z.literal("InputFileChanged"),
    payload: fileChangedSchema,
  }),
  z.object({
    kind: z.literal("InputAccepted"),
    payload: inputAcceptedSchema,
  }),
  z.object({
    kind: z.literal("InputError"),
    payload: inputErrorDTOSchema,
  }),
]);

export const externalEventSchema = z.object({
  level: eventLevelSchema,
  fields: externalEventsSchema,
});

export const clientEventSchema = z.object({
  kind: z.literal("Change"),
  payload: changeDTOSchema,
});
