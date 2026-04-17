import { html, LitElement } from "lit";
import { property } from "lit/decorators.js";
import { ServerDesc } from "@browsersync/generated/dto.js";
import { base } from "../../styles/base.css.js";

class BsServerDetail extends LitElement {
    @property({ type: Object })
    server: ServerDesc = { routes: [], id: "" };

    static styles = [base];

    render() {
        return html`
            <pre><code>${JSON.stringify(this.server, null, 2)}</code></pre>
        `;
    }
}

customElements.define("bs-server-detail", BsServerDetail);
