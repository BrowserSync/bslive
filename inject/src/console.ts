import {Subject} from "rxjs";
import {LogLevelDTO} from "@browsersync/generated/dto";

export interface ConsoleEvent {
  level: LogLevelDTO,
  text: string
}

export function createLRConsoleObserver(): [Subject<ConsoleEvent>, Pick<typeof console, "trace" | "debug" | "info" | "error">] {
  const subject = new Subject<ConsoleEvent>;
  return [subject, {
    debug: function (...data: any[]): void {
      subject.next({
        level: LogLevelDTO.Debug,
        text: data.join('\n')
      });
    },
    info: function (...data: any[]): void {
      subject.next({
        level: LogLevelDTO.Info,
        text: data.join('\n')
      });
    },
    trace: function (...data: any[]): void {
      subject.next({
        level: LogLevelDTO.Trace,
        text: data.join('\n')
      });
    },
    error: function (...data: any[]): void {
      subject.next({
        level: LogLevelDTO.Error,
        text: data.join('\n')
      });
    },
  }]
}