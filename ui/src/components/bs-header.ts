import { css, html, LitElement } from "lit";
import { property } from "lit/decorators.js";
import { ServerDTO } from "@browsersync/generated/dto";
import { base } from "../../styles/base.css";

class BsHeader extends LitElement {
    @property({ type: Object })
    servers: ServerDTO[] = [];

    static styles = [
        base,
        css`
            .logo {
                position: relative;
                color: var(--theme-txt-color);
            }

            .logo bs-icon::part(svg) {
                height: 30px;
                width: 140px;
            }
        `,
    ];

    render() {
        return html`
            <div class="logo">
                <bs-icon icon-name="wordmark"></bs-icon>
            </div>
        `;
    }
}

customElements.define("bs-header", BsHeader);
