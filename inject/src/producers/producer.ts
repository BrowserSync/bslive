import { ClientEvent, ConnectInfo } from "@browsersync/generated/dto";
import { Observable } from "rxjs";

export interface Producer {
    create(connectInfo: ConnectInfo): Observable<ClientEvent>;
}
