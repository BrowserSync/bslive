import { css, html, LitElement, PropertyValues } from "lit";
import { customElement, property } from "lit/decorators.js";
import { createRef, ref, Ref } from "lit/directives/ref.js";
import { base } from "../../styles/base.css.js";
import { tokens } from "../../styles/tokens.js";

@customElement("bs-token-env")
class TokenEnv extends LitElement {
    static styles = [base, tokens];

    render() {
        return html` <slot></slot> `;
    }
}
