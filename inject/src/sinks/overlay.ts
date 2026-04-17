import { Sink } from "./sink.js";
import { ClientConfigDTO, ClientEvent } from "@browsersync/generated/dto.js";
import { ConsoleApi } from "./console.js";
import {
    filter,
    ignoreElements,
    map,
    Observable,
    switchMap,
    tap,
    timer,
} from "rxjs";
import { overlay } from "../ui/overlay.js";

export const overlayPlugin: Sink<ClientEvent, [ConsoleApi]> = {
    name: "overlay plugin",
    globalSetup: (clientEvents$, log) => {
        return [clientEvents$, [log]];
    },
    resetSink(
        clientEvent$: Observable<ClientEvent>,
        api: [ConsoleApi],
        config: ClientConfigDTO,
    ): Observable<unknown> {
        const [log] = api;
        return clientEvent$.pipe(
            filter((x) => x.kind === "DisplayMessage"),
            map((x) => x.payload),
            switchMap((displayMessage) => {
                const cleanup = overlay({ displayMessage });
                return timer(2000).pipe(tap(() => cleanup()));
            }),
            ignoreElements(),
        );
    },
};
