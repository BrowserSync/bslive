import {css, html, LitElement} from "lit";
import {property} from "lit/decorators.js";
import {GetServersMessageResponse, ServerDesc, ServerDTO} from "../../../generated/dto";

class BsDebug extends LitElement {
  @property({type: Object})
  servers: GetServersMessageResponse = {servers: []};

  @property({type: Object})
  me: ServerDesc = {routes: [], id: ''}

  get otherServers(): ServerDTO[] {
    return this.servers.servers
      .filter(server => server.id !== this.me.id)
  }

  render() {
    return html`
        <bs-header></bs-header>
        <bs-server-detail .server=${this.me}></bs-server-detail>
        ${this.otherServers.length > 0 ? html`
            <bs-server-list .servers=${this.otherServers}></bs-server-list>` : null}
    `
  }
}

customElements.define('bs-debug', BsDebug)