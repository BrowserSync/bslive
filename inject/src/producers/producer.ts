import { ClientEvent, ConnectInfo } from "@browsersync/generated/dto.js";
import { Observable } from "rxjs";

export interface Producer {
    create(connectInfo: ConnectInfo): Observable<ClientEvent>;
}
