import { BsSystem } from "@browsersync/bslive";
const sys = new BsSystem();
const controller = new AbortController();
const done = sys.start(["bslive", "."], controller.signal);
setTimeout(() => {
    sys.stop();
    done.then((x) => console.log("result ->", x));
}, 1000);
