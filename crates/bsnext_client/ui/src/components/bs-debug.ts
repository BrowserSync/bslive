import {css, html, LitElement} from "lit";
import {property} from "lit/decorators.js";

class BsDebug extends LitElement {
  @property({ type: Object })
  data = {}

  static styles = [
    css``
  ]

  render() {
    return html`
    <pre><code>${JSON.stringify(this.data, null, 2)}</code></pre>
    `
  }
}

customElements.define('bs-debug', BsDebug)