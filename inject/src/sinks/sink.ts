import { Observable, Subject } from "rxjs";
import {
    ClientConfigDTO,
    ClientEvent,
    LogLevelDTO,
} from "@browsersync/generated/dto.js";
import { ConsoleApi } from "./console.js";

interface LogEffect {
    kind: "log";
    level: LogLevelDTO;
    args: any[];
}

// prettier-ignore
export interface Sink<T = any, API = any, U = any> {
    name: string;
    globalSetup: (
        events$: Observable<ClientEvent>,
        log: ConsoleApi
    ) => [Subject<T> | Observable<T>, API];

    resetSink(
        events: Observable<T>,
        api: API,
        config: ClientConfigDTO
    ): Observable<unknown>;
}
