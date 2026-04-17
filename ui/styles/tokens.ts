import { css } from "lit";

export const tokens = css`
    :host {
        --brand-blue: #0f2634;
        --brand-grey: #6d6d6d;
        --brand-red: #f24747;
        --brand-white: #ffffff;

        --theme-txt-color: var(--brand-blue);
        --theme-page-color: var(--brand-white);
        --theme-font-family: -apple-system, BlinkMacSystemFont, "Segoe UI",
            Roboto, Oxygen, Ubuntu, Cantarell, "Open Sans", "Helvetica Neue",
            sans-serif;
    }
`;
