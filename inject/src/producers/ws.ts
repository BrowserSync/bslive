import { Producer } from "./producer";
import { webSocket } from "rxjs/webSocket";
import { ClientEvent } from "@browsersync/generated/dto";
import { retry } from "rxjs";

export function ws(): Producer {
    return {
        create: (connectInfo) => {
            const current = new URL(window.location.href);

            // only use 'wss' protocol when we know it's safe to do so
            const ws_proto = current.protocol === "https:" ? "wss" : "ws";

            let ws_url;
            if (connectInfo.host) {
                ws_url = new URL(ws_proto + "://" + connectInfo.host);
            } else {
                const clone = new URL(current);
                clone.protocol = ws_proto;
                clone.pathname = "/__bs_ws";
                ws_url = clone;
            }

            const socket = webSocket<ClientEvent>(ws_url.toString());
            return socket.pipe(retry({ delay: 5000 }));
        },
    };
}
