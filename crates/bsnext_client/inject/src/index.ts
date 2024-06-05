// @ts-ignore
import {Reloader} from "livereload-js/src/reloader.js";
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

function changedPath(change: ChangeDTO) {
  switch (change.kind) {
    case "FsMany": {
      // todo: if this collection of events contains
      // anything that will cause a refresh, just do it immediately
      for (let changeDTO of change.payload) {
        changedPath(changeDTO);
      }
      break
    }
    case "Fs": {
      let path = change.payload.path;
      r.reload(path, {
        liveCSS: true,
        liveImg: true,
        reloadMissingCSS: true,
        originalPath: '',
        overrideURL: '',
        serverURL: ``,
      })
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

