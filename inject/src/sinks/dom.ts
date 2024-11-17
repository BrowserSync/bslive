import { filter, ignoreElements, map, Observable, tap } from "rxjs";
import { changeDTOSchema } from "@browsersync/generated/schema";
import { ClientConfigDTO, ClientEvent } from "@browsersync/generated/dto";
import { Sink } from "./sink";
import { Reloader } from "../../vendor/live-reload/src/reloader";
import { Timer } from "../../vendor/live-reload/src/timer";
import { ConsoleApi } from "./console";
import { changedPath } from "./dom/changed-path";

export const domPlugin: Sink<ClientEvent, [ConsoleApi, Reloader]> = {
  name: "dom plugin",
  globalSetup: (clientEvents$, log) => {
    const reloader = new Reloader(window, log, Timer);
    return [clientEvents$, [log, reloader]];
  },
  resetSink(
    clientEvent$: Observable<ClientEvent>,
    api: [ConsoleApi, Reloader],
    config: ClientConfigDTO,
  ): Observable<ClientEvent> {
    const [log, reloader] = api;
    return clientEvent$.pipe(
      filter((x) => x.kind === "Change"),
      map((x) => x.payload),
      tap((change) => {
        log.trace(
          "incoming message",
          JSON.stringify({ change, config }, null, 2),
        );
        const parsed = changeDTOSchema.parse(change);
        changedPath(parsed, log, reloader);
      }),
      ignoreElements(),
    );
  },
};
