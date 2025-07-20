// schema.ts
import { z } from "zod";

// dto.ts
var LogLevelDTO = /* @__PURE__ */ ((LogLevelDTO2) => {
  LogLevelDTO2["Info"] = "info";
  LogLevelDTO2["Debug"] = "debug";
  LogLevelDTO2["Trace"] = "trace";
  LogLevelDTO2["Error"] = "error";
  return LogLevelDTO2;
})(LogLevelDTO || {});
var ChangeKind = /* @__PURE__ */ ((ChangeKind2) => {
  ChangeKind2["Changed"] = "Changed";
  ChangeKind2["Added"] = "Added";
  ChangeKind2["Removed"] = "Removed";
  return ChangeKind2;
})(ChangeKind || {});
var EventLevel = /* @__PURE__ */ ((EventLevel2) => {
  EventLevel2["External"] = "BSLIVE_EXTERNAL";
  return EventLevel2;
})(EventLevel || {});

// schema.ts
var logLevelDTOSchema = z.nativeEnum(LogLevelDTO);
var clientConfigDTOSchema = z.object({
  log_level: logLevelDTOSchema
});
var connectInfoSchema = z.object({
  ws_path: z.string(),
  host: z.string().optional()
});
var debounceDTOSchema = z.object({
  kind: z.string(),
  ms: z.string()
});
var fileChangedDTOSchema = z.object({
  path: z.string()
});
var filesChangedDTOSchema = z.object({
  paths: z.array(z.string())
});
var serverIdentityDTOSchema = z.discriminatedUnion("kind", [
  z.object({
    kind: z.literal("Both"),
    payload: z.object({
      name: z.string(),
      bind_address: z.string()
    })
  }),
  z.object({
    kind: z.literal("Address"),
    payload: z.object({
      bind_address: z.string()
    })
  }),
  z.object({
    kind: z.literal("Named"),
    payload: z.object({
      name: z.string()
    })
  }),
  z.object({
    kind: z.literal("Port"),
    payload: z.object({
      port: z.number()
    })
  }),
  z.object({
    kind: z.literal("PortNamed"),
    payload: z.object({
      port: z.number(),
      name: z.string()
    })
  })
]);
var serverDTOSchema = z.object({
  id: z.string(),
  identity: serverIdentityDTOSchema,
  socket_addr: z.string()
});
var getActiveServersResponseDTOSchema = z.object({
  servers: z.array(serverDTOSchema)
});
var injectConfigSchema = z.object({
  connect: connectInfoSchema,
  ctx_message: z.string()
});
var inputAcceptedDTOSchema = z.object({
  path: z.string()
});
var sseDTOOptsSchema = z.object({
  body: z.string()
});
var routeKindDTOSchema = z.discriminatedUnion("kind", [
  z.object({
    kind: z.literal("Html"),
    payload: z.object({
      html: z.string()
    })
  }),
  z.object({
    kind: z.literal("Json"),
    payload: z.object({
      json_str: z.string()
    })
  }),
  z.object({
    kind: z.literal("Raw"),
    payload: z.object({
      raw: z.string()
    })
  }),
  z.object({
    kind: z.literal("Sse"),
    payload: z.object({
      sse: sseDTOOptsSchema
    })
  }),
  z.object({
    kind: z.literal("Proxy"),
    payload: z.object({
      proxy: z.string()
    })
  }),
  z.object({
    kind: z.literal("Dir"),
    payload: z.object({
      dir: z.string(),
      base: z.string().optional()
    })
  })
]);
var serverChangeSchema = z.discriminatedUnion("kind", [
  z.object({
    kind: z.literal("Stopped"),
    payload: z.object({
      bind_address: z.string()
    })
  }),
  z.object({
    kind: z.literal("Started"),
    payload: z.undefined().optional()
  }),
  z.object({
    kind: z.literal("Patched"),
    payload: z.undefined().optional()
  }),
  z.object({
    kind: z.literal("Errored"),
    payload: z.object({
      error: z.string()
    })
  })
]);
var serverChangeSetItemSchema = z.object({
  identity: serverIdentityDTOSchema,
  change: serverChangeSchema
});
var serverChangeSetSchema = z.object({
  items: z.array(serverChangeSetItemSchema)
});
var routeDTOSchema = z.object({
  path: z.string(),
  kind: routeKindDTOSchema
});
var serversChangedDTOSchema = z.object({
  servers_resp: getActiveServersResponseDTOSchema
});
var stderrLineDTOSchema = z.object({
  line: z.string(),
  prefix: z.string().optional()
});
var stdoutLineDTOSchema = z.object({
  line: z.string(),
  prefix: z.string().optional()
});
var stoppedWatchingDTOSchema = z.object({
  paths: z.array(z.string())
});
var watchingDTOSchema = z.object({
  paths: z.array(z.string()),
  debounce: debounceDTOSchema
});
var changeKindSchema = z.nativeEnum(ChangeKind);
var changeDTOSchema = z.lazy(
  () => z.discriminatedUnion("kind", [
    z.object({
      kind: z.literal("Fs"),
      payload: z.object({
        path: z.string(),
        change_kind: changeKindSchema
      })
    }),
    z.object({
      kind: z.literal("FsMany"),
      payload: z.array(changeDTOSchema)
    })
  ])
);
var clientEventSchema = z.discriminatedUnion("kind", [
  z.object({
    kind: z.literal("Change"),
    payload: changeDTOSchema
  }),
  z.object({
    kind: z.literal("WsConnection"),
    payload: clientConfigDTOSchema
  }),
  z.object({
    kind: z.literal("Config"),
    payload: clientConfigDTOSchema
  })
]);
var eventLevelSchema = z.nativeEnum(EventLevel);
var outputLineDTOSchema = z.discriminatedUnion("kind", [
  z.object({
    kind: z.literal("Stdout"),
    payload: stdoutLineDTOSchema
  }),
  z.object({
    kind: z.literal("Stderr"),
    payload: stderrLineDTOSchema
  })
]);
var inputErrorDTOSchema = z.discriminatedUnion("kind", [
  z.object({
    kind: z.literal("MissingInputs"),
    payload: z.string()
  }),
  z.object({
    kind: z.literal("InvalidInput"),
    payload: z.string()
  }),
  z.object({
    kind: z.literal("NotFound"),
    payload: z.string()
  }),
  z.object({
    kind: z.literal("InputWriteError"),
    payload: z.string()
  }),
  z.object({
    kind: z.literal("PathError"),
    payload: z.string()
  }),
  z.object({
    kind: z.literal("PortError"),
    payload: z.string()
  }),
  z.object({
    kind: z.literal("DirError"),
    payload: z.string()
  }),
  z.object({
    kind: z.literal("YamlError"),
    payload: z.string()
  }),
  z.object({
    kind: z.literal("MarkdownError"),
    payload: z.string()
  }),
  z.object({
    kind: z.literal("HtmlError"),
    payload: z.string()
  }),
  z.object({
    kind: z.literal("Io"),
    payload: z.string()
  }),
  z.object({
    kind: z.literal("UnsupportedExtension"),
    payload: z.string()
  }),
  z.object({
    kind: z.literal("MissingExtension"),
    payload: z.string()
  }),
  z.object({
    kind: z.literal("EmptyInput"),
    payload: z.string()
  }),
  z.object({
    kind: z.literal("BsLiveRules"),
    payload: z.string()
  })
]);
var internalEventsDTOSchema = z.discriminatedUnion("kind", [
  z.object({
    kind: z.literal("ServersChanged"),
    payload: getActiveServersResponseDTOSchema
  }),
  z.object({
    kind: z.literal("TaskReport"),
    payload: z.object({
      id: z.string()
    })
  })
]);
var startupEventDTOSchema = z.discriminatedUnion("kind", [
  z.object({
    kind: z.literal("Started"),
    payload: z.undefined().optional()
  }),
  z.object({
    kind: z.literal("FailedStartup"),
    payload: z.string()
  })
]);
var serverDescSchema = z.object({
  routes: z.array(routeDTOSchema),
  id: z.string()
});
var externalEventsDTOSchema = z.discriminatedUnion("kind", [
  z.object({
    kind: z.literal("ServersChanged"),
    payload: serversChangedDTOSchema
  }),
  z.object({
    kind: z.literal("Watching"),
    payload: watchingDTOSchema
  }),
  z.object({
    kind: z.literal("WatchingStopped"),
    payload: stoppedWatchingDTOSchema
  }),
  z.object({
    kind: z.literal("FileChanged"),
    payload: fileChangedDTOSchema
  }),
  z.object({
    kind: z.literal("FilesChanged"),
    payload: filesChangedDTOSchema
  }),
  z.object({
    kind: z.literal("InputFileChanged"),
    payload: fileChangedDTOSchema
  }),
  z.object({
    kind: z.literal("InputAccepted"),
    payload: inputAcceptedDTOSchema
  }),
  z.object({
    kind: z.literal("OutputLine"),
    payload: outputLineDTOSchema
  })
]);
export {
  changeDTOSchema,
  changeKindSchema,
  clientConfigDTOSchema,
  clientEventSchema,
  connectInfoSchema,
  debounceDTOSchema,
  eventLevelSchema,
  externalEventsDTOSchema,
  fileChangedDTOSchema,
  filesChangedDTOSchema,
  getActiveServersResponseDTOSchema,
  injectConfigSchema,
  inputAcceptedDTOSchema,
  inputErrorDTOSchema,
  internalEventsDTOSchema,
  logLevelDTOSchema,
  outputLineDTOSchema,
  routeDTOSchema,
  routeKindDTOSchema,
  serverChangeSchema,
  serverChangeSetItemSchema,
  serverChangeSetSchema,
  serverDTOSchema,
  serverDescSchema,
  serverIdentityDTOSchema,
  serversChangedDTOSchema,
  sseDTOOptsSchema,
  startupEventDTOSchema,
  stderrLineDTOSchema,
  stdoutLineDTOSchema,
  stoppedWatchingDTOSchema,
  watchingDTOSchema
};
