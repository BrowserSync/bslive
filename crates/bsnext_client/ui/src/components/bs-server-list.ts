import {css, html, LitElement} from "lit";
import {property} from "lit/decorators.js";
import {ServerDTO} from "../../../generated/dto";
import { base } from "../../styles/base.css";

class BsServerList extends LitElement {
  @property({type: Object})
  servers: ServerDTO[] = []

  static styles = [
    base,
    css`
    
    `
  ]

  render() {
    return html`
        ${this.servers
                .map(server => {
                    const display_addr = 'http://' + server.socket_addr;
                    let url = new URL(display_addr);
                    let bs_url = new URL('./__bslive', display_addr);
                    return html`
                        <div>
                            <bs-server-identity .identity=${server.identity}></bs-server-identity>
                            <p><a href=${url} target="_blank"><code>${url}</code></a></p>
                            <p>
                                <bs-icon icon-name="logo"></bs-icon>
                                <small><a href=${bs_url} target="_blank"><code>${bs_url}</code></a></small>
                            </p>
                        </div>
                    `
                })}
    `
  }
}

customElements.define('bs-server-list', BsServerList)