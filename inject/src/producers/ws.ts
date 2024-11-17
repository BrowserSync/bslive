import { Producer } from "./producer";
import { webSocket } from "rxjs/webSocket";
import { ClientEvent } from "@browsersync/generated/dto";
import { retry } from "rxjs";

export function ws(): Producer {
  return {
    create: () => {
      const url = new URL(window.location.href);

      url.protocol = url.protocol === "http:" ? "ws" : "wss";
      url.pathname = "/__bs_ws";

      const socket = webSocket<ClientEvent>(url.origin + url.pathname);
      return socket.pipe(retry({ delay: 5000 }));
    },
  };
}
