import {Reloader} from "../vendor/live-reload/src/reloader";
import {Timer} from "../vendor/live-reload/src/timer";

import {webSocket} from "rxjs/webSocket";
import {filter, map, retry, withLatestFrom} from "rxjs";
import {clientEventSchema} from "../../generated/schema.js";
import {ChangeDTO, ClientConfigDTO, ClientEvent, LogLevelDTO} from "@browsersync/generated/dto";
import {createLRConsoleObserver} from "./console";

const [consoleSubject, consoleApi] = createLRConsoleObserver();

const r = new Reloader(window, consoleApi, Timer);
const url = new URL(window.location.href);
url.protocol = url.protocol === 'http:' ? 'ws' : 'wss';
url.pathname = '/__bs_ws'
const socket = webSocket<ClientEvent>(url.origin + url.pathname);

const sub$ = socket.pipe(retry({delay: 5000}));
const change$ = sub$.pipe(filter(x => x.kind === "Change"));
const config$ = sub$.pipe(filter(x => x.kind === "Config"), map(x => x.payload));

change$.pipe(withLatestFrom(config$)).subscribe(([change, config]) => {
  consoleApi.trace("incoming message", JSON.stringify({change, config}, null, 2))
  const parsed = clientEventSchema.parse(change);
  switch (parsed.kind) {
    case "Change": {
      changedPath(parsed.payload);
      break;
    }
    default: {
      console.warn("unhandled client event")
    }
  }
});

// todo: the checks are lifted directly from live reload, we should not use them, but are a good starting point
const IMAGES_REGEX = /\.(jpe?g|png|gif|svg)$/i;

function changedPath(change: ChangeDTO) {
  switch (change.kind) {
    case "FsMany": {

      const hasNoneInjectable = change.payload.some(changeDTO => {
        switch (changeDTO.kind) {
          case "Fs":
            if (changeDTO.payload.path.match(/\.css(?:\.map)?$/i)) {
              return false
            }
            if (changeDTO.payload.path.match(IMAGES_REGEX)) {
              return false
            }

            // if we get here, we're not going to live inject anything
            return true
          case "FsMany":
            throw new Error("unreachable")
        }
      });

      // if any path will cause a reload anyway, don't both hot-reloading anything.
      if (hasNoneInjectable) {
        if (window.__playwright?.record) {
          return window.__playwright?.record({
            kind: 'reloadPage',
          })
        } else {
          return r.reloadPage()
        }
      }

      // if we get here, every path given was injectable, so try to inject them all
      for (let changeDTO of change.payload) {
        changedPath(changeDTO);
      }
      break
    }
    case "Fs": {
      let path = change.payload.path;
      const opts = {
        liveCSS: true,
        liveImg: true,
        reloadMissingCSS: true,
        originalPath: '',
        overrideURL: '',
        serverURL: ``,
      }
      if (window.__playwright?.record) {
        window.__playwright?.record({
          kind: 'reload',
          args: {
            path, opts
          }
        })
      } else {
        consoleApi.trace("will reload a file with path ", path)
        r.reload(path, opts)
      }
    }
  }
}

consoleSubject.pipe(withLatestFrom(config$)).subscribe(([evt, config]) => {
  const levelOrder = [LogLevelDTO.Trace, LogLevelDTO.Debug, LogLevelDTO.Info, LogLevelDTO.Error];
  const currentLevelIndex = levelOrder.indexOf(evt.level);
  const configLevelIndex = levelOrder.indexOf(config.log_level);

  if (currentLevelIndex >= configLevelIndex) {
    console.log(`[${evt.level}] ${evt.text}`);
  }
})

export function logMessage(message: string, level: LogLevelDTO, config: ClientConfigDTO): void {

}

// todo: share this with tests
declare global {
  interface Window {
    __playwright?: {
      calls?: any[],
      record?: (...args: any[]) => void
    }
  }
}