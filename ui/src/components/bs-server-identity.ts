import {html, LitElement} from "lit";
import {property} from "lit/decorators.js";
import {ServerIdentityDTO} from "@browsersync/generated/dto";
import {base} from "../../styles/base.css";

class BsServerIdentity extends LitElement {
  @property({type: Object})
  identity!: ServerIdentityDTO

  static styles = [base];

  render() {
    switch (this.identity.kind) {
      case "Named":
      case "Both": {
        return html`<p><strong>[named] ${this.identity.payload.name}</strong></p>`
      }
      default:
        return html`<p><strong>[unnamed]</strong></p>`
    }
  }
}

customElements.define('bs-server-identity', BsServerIdentity)