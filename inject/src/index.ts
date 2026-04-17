import { filter, map, merge, Observable, share, switchMap } from "rxjs";
import { Producer } from "./producers/producer.js";
import { ws } from "./producers/ws.js";
import { consolePlugin, NULL_CONSOLE } from "./sinks/console.js";
import { domPlugin } from "./sinks/dom.js";
import { InjectConfig } from "@browsersync/generated/dto.js";
import { injectConfigSchema } from "@browsersync/generated/schema.js";
import { overlayPlugin } from "./sinks/overlay.js";

((injectConfig) => {
    injectConfigSchema.parse(injectConfig);

    const producer: Producer = ws();
    const clientEvent$ = producer.create(injectConfig.connect);

    const [logEvent$, log] = consolePlugin.globalSetup(
        clientEvent$,
        NULL_CONSOLE,
    );
    const [domEvents$, domApis] = domPlugin.globalSetup(clientEvent$, log);

    // prettier-ignore
    const connection$ = clientEvent$.pipe(
        filter((x) => x.kind === "WsConnection"),
        map((x) => x.payload),
        share()
    );

    // prettier-ignore
    const config$ = clientEvent$.pipe(
        filter((x) => x.kind === "Config"),
        map((x) => x.payload)
    );

    /**
     * Side effects - this is where we react to incoming WS events
     */
    merge(config$, connection$)
        .pipe(
            switchMap((config) => {
                const sinks: Observable<unknown>[] = [
                    domPlugin.resetSink(domEvents$, domApis, config),
                    consolePlugin.resetSink(logEvent$, log, config),
                    overlayPlugin.resetSink(clientEvent$, [log], config),
                ];
                return merge(...sinks);
            }),
        )
        .subscribe();

    connection$.subscribe((config) => {
        log.info("🟢 Browsersync Live connected", { config });
    });
})(window.$BSLIVE_INJECT_CONFIG$);

export {};

// todo: share this with tests
declare global {
    interface Window {
        __playwright?: {
            calls?: any[];
            record?: (...args: any[]) => void;
        };
        $BSLIVE_INJECT_CONFIG$: InjectConfig;
    }
}
