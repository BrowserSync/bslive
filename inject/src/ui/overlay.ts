import { html, render } from "lit";
import { DisplayMessageDTO } from "@browsersync/generated/dto.js";
import "@browsersync/bslive-ui/components/bs-overlay.js";
import "@browsersync/bslive-ui/components/bs-panel.js";
import "@browsersync/bslive-ui/components/bs-token-env.js";

type OverlayCleanup = () => void;

export function overlay({
    displayMessage,
}: {
    displayMessage: DisplayMessageDTO;
}): OverlayCleanup {
    const closed = () => {
        /* noop */
    };
    let item = html`
        <style>
            bs-overlay::part(footer) {
                display: none;
            }
        </style>
        <bs-token-env>
            <bs-overlay @closed=${closed}>
                <bs-panel title="${displayMessage.message}" slot="content">
                    ${displayMessage.reason
                        ? html`<pre
                              slot="detail"
                          ><code>${displayMessage.reason}</code></pre>`
                        : null}
                </bs-panel>
            </bs-overlay>
        </bs-token-env>
    `;
    const backdrop = document.createElement("bs-overlay");
    render(item, document.body);
    return () => {
        if (backdrop.isConnected) {
            backdrop.remove();
        }
    };
}
