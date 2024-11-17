// todo: the checks are lifted directly from live reload, we should not use them, but are a good starting point
import { ChangeDTO } from "@browsersync/generated/dto";
import { ConsoleApi } from "../console";
import { Reloader } from "../../../vendor/live-reload/src/reloader";

export const IMAGES_REGEX = /\.(jpe?g|png|gif|svg)$/i;

export function changedPath(change: ChangeDTO, log: ConsoleApi, api: Reloader) {
  switch (change.kind) {
    case "FsMany": {
      const hasNoneInjectable = change.payload.some((changeDTO) => {
        switch (changeDTO.kind) {
          case "Fs":
            if (changeDTO.payload.path.match(/\.css(?:\.map)?$/i)) {
              return false;
            }
            if (changeDTO.payload.path.match(IMAGES_REGEX)) {
              return false;
            }

            // if we get here, we're not going to live inject anything
            return true;
          case "FsMany":
            throw new Error("unreachable");
        }
      });

      // if any path will cause a reload anyway, don't both hot-reloading anything.
      if (hasNoneInjectable) {
        if (window.__playwright?.record) {
          return window.__playwright?.record({
            kind: "reloadPage",
          });
        } else {
          return api.reloadPage();
        }
      }

      // if we get here, every path given was injectable, so try to inject them all
      for (let changeDTO of change.payload) {
        changedPath(changeDTO, log, api);
      }
      break;
    }
    case "Fs": {
      let path = change.payload.path;
      const opts = {
        liveCSS: true,
        liveImg: true,
        reloadMissingCSS: true,
        originalPath: "",
        overrideURL: "",
        serverURL: ``,
      };
      if (window.__playwright?.record) {
        window.__playwright?.record({
          kind: "reload",
          args: {
            path,
            opts,
          },
        });
      } else {
        log.trace("will reload a file with path ", path);
        api.reload(path, opts);
      }
    }
  }
}
