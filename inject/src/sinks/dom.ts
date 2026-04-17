import { filter, ignoreElements, map, Observable, tap } from "rxjs";
import { changeDTOSchema } from "@browsersync/generated/schema.js";
import { ClientConfigDTO, ClientEvent } from "@browsersync/generated/dto.js";
import { Sink } from "./sink.js";
import { Reloader } from "../../vendor/live-reload/src/reloader.js";
import { Timer } from "../../vendor/live-reload/src/timer.js";
import { ConsoleApi } from "./console.js";
import { changedPath } from "./dom/changed-path.js";

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
    ): Observable<unknown> {
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
