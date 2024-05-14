import {Observable, Subject} from "rxjs";

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

export function createLRConsoleObserver(): [Subject<ConsoleEvent>, Pick<typeof console, "log">] {
  const subject = new Subject<ConsoleEvent>;
  return [subject, {
    log: function (...data: any[]): void {
      subject.next({
        level: Level.Debug,
        text: data.join('\n')
      });
    },
  }]
}