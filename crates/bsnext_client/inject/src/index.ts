// @ts-ignore
import {Reloader,} from "livereload-js/src/reloader.js";
// @ts-ignore
import {Timer} from "livereload-js/src/timer.js";

import {webSocket} from "rxjs/webSocket";
import {retry} from "rxjs";
import {clientEventSchema} from "../../generated/schema.js";
import {ChangeDTO, ClientEvent} from "../../generated/dto";
import {createLRConsoleObserver, Level} from "./console";

const [consoleSubject, consoleApi] = createLRConsoleObserver();

const r = new Reloader(window, consoleApi, Timer);
const url = new URL(window.location.href);
url.protocol = url.protocol === 'http:' ? 'ws' : 'wss';
url.pathname = '/__bs_ws'
const socket = webSocket<ClientEvent>(url.origin + url.pathname);

socket
  .pipe(retry({delay: 5000}))
  .subscribe(m => {
    console.log(JSON.stringify(m, null, 2))
    const parsed = clientEventSchema.parse(m);
    switch (parsed.kind) {
      case "Change": {
        changedPath(parsed.payload);
        break;
      }
      default: {
        console.warn("unhandled client event")
      }
    }
  })

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
        r.reload(path, opts)
      }
    }
  }
}

consoleSubject.subscribe(evt => {
  switch (evt.level) {
    case Level.Trace:
      break;
    case Level.Debug:
      console.log('[debug]', evt.text)
      break;
    case Level.Info:
      break;
    case Level.Error:
      break;
  }
})

// todo: share this with tests
declare global {
  interface Window {
    __playwright?: {
      calls?: any[],
      record?: (...args: any[]) => void
    }
  }
}