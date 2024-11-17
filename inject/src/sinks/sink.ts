import { Observable, Subject } from "rxjs";
import { ClientConfigDTO, ClientEvent } from "@browsersync/generated/dto";
import { ConsoleApi } from "./console";

export interface Sink<T = any, API = any, U = any> {
  name: string;
  globalSetup: (
    events$: Observable<ClientEvent>,
    log: ConsoleApi,
  ) => [Subject<T> | Observable<T>, API];

  resetSink(
    events: Observable<T>,
    api: API,
    config: ClientConfigDTO,
  ): Observable<U>;
}
