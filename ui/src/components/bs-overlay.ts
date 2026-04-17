import { css, html, LitElement, PropertyValues } from "lit";
import { customElement, property } from "lit/decorators.js";
import { createRef, ref, Ref } from "lit/directives/ref.js";
import { base } from "../../styles/base.css.js";
import { tokens } from "../../styles/tokens.js";

const WIDTH_NARROW = "narrow";
const WIDTH_WIDE = "wide";

@customElement("bs-overlay")
class Overlay extends LitElement {
    static styles = [
        tokens,
        base,
        css`
            ::backdrop {
                background: rgba(0, 0, 0, 0.7);
            }

            dialog {
                max-height: 90vh;
                max-width: 90vw;
            }

            dialog[data-variant="narrow"] {
                width: 550px;

                button[value="narrow"] {
                    display: none;
                }
            }

            dialog[data-variant="wide"] {
                width: 90vw;

                button[value="wide"] {
                    display: none;
                }
            }

            button {
            }

            .footer {
                display: flex;
                margin-top: 12px;
                justify-content: flex-end;
                gap: 0.5rem;
            }
        `,
    ];

    @property({ type: String })
    kind: "overlay" | "inline" = "overlay";

    dialogRef: Ref<HTMLDialogElement> = createRef();

    protected firstUpdated(_changedProperties: PropertyValues) {
        super.firstUpdated(_changedProperties);
        this.dialogRef.value?.showModal();
    }

    closed() {
        this.dispatchEvent(
            new Event("closed", { bubbles: true, composed: true }),
        );
    }

    @property({ type: String })
    width: "narrow" | "wide" = "narrow";

    toggleWidth(evt: MouseEvent) {
        if (evt.currentTarget instanceof HTMLButtonElement) {
            const val = evt.currentTarget.value;
            if (val === WIDTH_WIDE || val === WIDTH_NARROW) {
                this.width = val;
            }
        }
    }

    render() {
        return html`
            <dialog
                id="my-dialog"
                ${ref(this.dialogRef)}
                @close=${this.closed}
                data-variant=${this.width}
            >
                <slot name="content"></slot>
                <div class="footer" part="footer">
                    <button
                        @click=${this.toggleWidth}
                        value=${WIDTH_NARROW}
                        part="width-toggle"
                    >
                        ↔️ Narrow
                    </button>
                    <button
                        @click=${this.toggleWidth}
                        value=${WIDTH_WIDE}
                        part="width-toggle"
                    >
                        ↕️ Wide
                    </button>
                    <button
                        commandfor="my-dialog"
                        command="close"
                        part="button"
                    >
                        ✕ Close
                    </button>
                </div>
            </dialog>
        `;
    }
}
