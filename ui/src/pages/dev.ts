import { css, html, LitElement } from "lit";
import { customElement, property } from "lit/decorators.js";
import { base } from "../../styles/base.css.js";
import { LARGE_CODE } from "../fixtures/text.js";

@customElement("bs-dev-page")
class DevPage extends LitElement {
    static styles = [
        base,
        css`
            .stack {
                display: grid;
                width: 100%;
                grid-row-gap: 0.5rem;
            }
            bs-icon[icon-name="wordmark"]::part(svg) {
                height: 30px;
                width: 140px;
            }
            ::slotted {
                border: 5px solid green;
            }
            bs-overlay {
                max-width: 200px;
            }
        `,
    ];

    @property({ type: Boolean })
    modalShowing = new URLSearchParams(location.search).has("showModal");

    @property({ type: Boolean })
    timedModalShowing = new URLSearchParams(location.search).has(
        "showTimedModal",
    );

    showTimedModal() {
        this.timedModalShowing = true;
        clearTimeout(this._timer);
        this._timer = setTimeout(() => {
            this.timedModalShowing = false;
            this.requestUpdate();
        }, 2000) as unknown as number;
    }

    _timer: number | undefined = undefined;

    showModal() {
        this.modalShowing = true;
    }

    closed() {
        this.modalShowing = false;
    }

    get autoOpenLink() {
        const url = new URL(location.href);
        url.searchParams.set("showModal", "true");
        return url.toString();
    }

    render() {
        return html`
            <h2>Overlays</h2>
            <div class="stack">
                <div>
                    <button @click=${this.showModal}>Show modal</button>
                    <a href="${this.autoOpenLink}">Auto open</a>
                </div>
                <div>
                    <button @click=${this.showTimedModal}>
                        Show timed modal
                    </button>
                </div>
                <bs-panel title="Single line">
                    <pre
                        slot="detail"
                    ><code>A problem has occured in file <b>index.html</b></code></pre>
                </bs-panel>
                <bs-panel title="With detail">
                    <pre slot="detail"><code>${LARGE_CODE}</code></pre>
                </bs-panel>
                <bs-panel title="without any detail"></bs-panel>
            </div>
            <h2>Icons</h2>
            <p>
                <bs-icon icon-name="logo"></bs-icon>
            </p>
            <p>
                <bs-icon icon-name="wordmark"></bs-icon>
            </p>

            ${this.modalShowing
                ? html`<bs-overlay title="Single line" @closed=${this.closed}>
                      <bs-panel title="Single line" slot="content">
                          <pre slot="detail">${LARGE_CODE}</pre>
                      </bs-panel>
                  </bs-overlay>`
                : null}
            ${this.timedModalShowing
                ? html`<bs-overlay @closed=${this.closed}>
                      <bs-panel title="Timed Modal" slot="content">
                          <pre
                              slot="detail"
                          ><code>A problem has occured in file <b>index.html</b></code></pre>
                      </bs-panel>
                  </bs-overlay>`
                : null}
        `;
    }
}
