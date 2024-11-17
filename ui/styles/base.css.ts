import { css } from "lit";

export const base = css`
  pre {
    margin: 0;
  }
  a {
    color: var(--theme-txt-color);
    &:hover {
      text-decoration: none;
    }
  }
  p {
    margin: 0;
    padding: 0;
  }
`;
