import { ClientEvent } from "@browsersync/generated/dto";
import { Observable } from "rxjs";

export interface Producer {
  create(): Observable<ClientEvent>;
}
