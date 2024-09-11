import {Subject} from "rxjs";

export enum Level {
  Trace = "Trace",
  Debug = "Debug",
  Info = "Info",
  Error = "Error",
}

export interface ConsoleEvent {
  level: Level,
  text: string
}

export function createLRConsoleObserver(): [Subject<ConsoleEvent>, Pick<typeof console, "trace" | "debug" | "info" | "error">] {
  const subject = new Subject<ConsoleEvent>;
  return [subject, {
    debug: function (...data: any[]): void {
      subject.next({
        level: Level.Debug,
        text: data.join('\n')
      });
    },
    info: function (...data: any[]): void {
      subject.next({
        level: Level.Info,
        text: data.join('\n')
      });
    },
    trace: function (...data: any[]): void {
      subject.next({
        level: Level.Trace,
        text: data.join('\n')
      });
    },
    error: function (...data: any[]): void {
      subject.next({
        level: Level.Error,
        text: data.join('\n')
      });
    },
  }]
}