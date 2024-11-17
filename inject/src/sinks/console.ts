import {
  ClientConfigDTO,
  ClientEvent,
  LogLevelDTO,
} from "@browsersync/generated/dto";
import { Observable, Subject, tap } from "rxjs";
import { Sink } from "./sink";
import { undefined } from "zod";

export interface ConsoleEvent {
  level: LogLevelDTO;
  args: any[];
}

export type ConsoleApi = Pick<
  typeof console,
  "trace" | "debug" | "info" | "error"
>;

export const NULL_CONSOLE: ConsoleApi = {
  debug(...data: any[]): void {},
  error(...data: any[]): void {},
  info(...data: any[]): void {},
  trace(...data: any[]): void {},
};

export const consolePlugin: Sink<ConsoleEvent, ConsoleApi> = {
  name: "console",
  globalSetup: (events) => {
    const subject = new Subject<ConsoleEvent>();
    const api: ConsoleApi = {
      debug: function (...data: any[]): void {
        subject.next({
          level: LogLevelDTO.Debug,
          args: data,
        });
      },
      info: function (...data: any[]): void {
        subject.next({
          level: LogLevelDTO.Info,
          args: data,
        });
      },
      trace: function (...data: any[]): void {
        subject.next({
          level: LogLevelDTO.Trace,
          args: data,
        });
      },
      error: function (...data: any[]): void {
        subject.next({
          level: LogLevelDTO.Error,
          args: data,
        });
      },
    };
    return [subject, api];
  },
  resetSink: (
    events$: Observable<ConsoleEvent>,
    api: ConsoleApi,
    config: ClientConfigDTO,
  ): Observable<unknown> => {
    return events$.pipe(
      tap((evt) => {
        const levelOrder = [
          LogLevelDTO.Trace,
          LogLevelDTO.Debug,
          LogLevelDTO.Info,
          LogLevelDTO.Error,
        ];
        const currentLevelIndex = levelOrder.indexOf(evt.level);
        const configLevelIndex = levelOrder.indexOf(config.log_level);

        if (currentLevelIndex >= configLevelIndex) {
          console.log(`[${evt.level}]`, ...evt.args);
        }
      }),
    );
  },
};
