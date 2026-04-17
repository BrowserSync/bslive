import { css, html, LitElement } from "lit";
import { customElement, property } from "lit/decorators.js";
import { base } from "../../styles/base.css.js";
import { logo } from "./bs-icon.js";
import { tokens } from "../../styles/tokens.js";

@customElement("bs-panel")
class Panel extends LitElement {
    static styles = [
        tokens,
        base,
        css`
            .root {
                background: white;
                color: var(--brand-blue);
                display: grid;
                grid-template-columns: 100%;
                grid-row-gap: 0.6rem;
            }
            slot:has(*) {
                display: block;
                outline: 5px solid red;
            }
            .header {
                display: flex;
                align-items: center;
                gap: 0.4rem;
            }
            ::slotted(*) {
                max-width: 100%;
                overflow-x: auto;
            }
            bs-icon {
                --bs-icon-height: 24px;
                --bs-icon-width: 24px;
                --bs-icon-color: var(--brand-red);
                display: inline-block;
                color: var(--brand-red);
                position: relative;
                top: -2px;
            }
            .title {
                font-size: 0.8rem;
                font-family: var(--theme-font-family);
            }
            slot::slotted(*) {
                font-size: 10px;
            }
        `,
    ];
    @property({ type: String })
    title = "...";
    render() {
        return html`<div class="root">
            <header class="header">
                ${logo()}
                <span class="title">
                    ${this.title}
                <span>
            </header>
            <slot name="detail"></slot></code>
        </div>`;
    }
}
