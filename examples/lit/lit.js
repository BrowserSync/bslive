import {LitElement, html, css, render} from "lit"

customElements.define('a-btn', class extends LitElement {
    static properties = {
        variant: {type: String}
    }
    static styles = css`
        [data-variant="primary"] {
            --button-bg: darkblue;
        }

        [name=icon]::slotted(span) {
            color: black;
            font-weight: bold;
            outline: 1px dotted black;
        }

        button {
            background: var(--button-bg, darkblue);
        }
    `

    constructor() {
        super();
        this.variant = "secondary"
    }

    render() {
        return html`
            <button part="button" data-variant=${this.variant} @click=${() => this.dispatchEvent(new Event("click"))}>
                <slot name="icon" part="icon left"></slot>
                <slot></slot>
                <slot name="arrow" part="icon right"></slot>
            </button>`
    }
});
customElements.define('a-b', class extends LitElement {
    static properties = {
        state: {type: Object}
    }

    constructor() {
        super();
        this.state = {};
    }

    static styles = css`
        :host {
            --btn-txt-color: green;
            --button-bg: green;
        }

        a-btn::part(button) {
            font-weight: bold;
        }

        a-btn::part(icon left) {
            font-size: 0.5em;
        }

        a-btn::part(icon) {
            font-size: 2em;
        }

    `


    render() {
        return html`
            <pre>${JSON.stringify(this.state)}</pre>
            <a-btn .variant=${"secondary"} @click=${() => this.state = {c: "d"}}>
                <span slot="icon">✆</span>
                <span slot="arrow">📲</span>
                Click me
            </a-btn>
        `
    }
})
let state = {a: "b"}
let app = html`
    <a-b .state=${state}></a-b>`;
render(app, document.querySelector('main'))
