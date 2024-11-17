import {css, html, LitElement} from "lit";
import {property} from "lit/decorators.js";
import {ServerDesc, ServerDTO} from "@browsersync/generated/dto";
import {base} from "../../styles/base.css";

class BsServerDetail extends LitElement {
  @property({type: Object})
  server: ServerDesc = {routes: [], id: ''}

  static styles = [
    base
  ]

  render() {
    return html`
        <pre><code>${JSON.stringify(this.server, null, 2)}</code></pre>
    `
  }
}

customElements.define('bs-server-detail', BsServerDetail)