import { BsSystem } from "./index.js";

export function create() {
    return new (class {
        /** @type {Promise<any>} */
        prom;

        fromArgs(args) {
            let sys = new BsSystem();
            const inputArgs = ["bslive", ...args];
            const controller = new AbortController();

            this.prom = sys.start(inputArgs, controller.signal);
            this.prom.catch(() => console.error("huh?"));

            return new (class {
                stop() {
                    controller.abort();
                    sys.stop();
                }
            })();
        }
    })();
}
