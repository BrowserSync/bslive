import { css, html, LitElement } from "lit";
import { property } from "lit/decorators.js";
import { base } from "../../styles/base.css";

class BsIcon extends LitElement {
  @property({ type: String, attribute: "icon-name" })
  iconName!: string;

  static styles = [
    base,
    css`
      .svg-icon {
        display: inline-block;
        fill: var(--bs-icon-color, currentColor);
        height: var(--bs-icon-height, 1em);
        width: var(--bs-icon-width, 1em);
        vertical-align: middle;
      }
    `,
  ];

  get icon() {
    switch (this.iconName) {
      case "logo":
        return html` <svg class="svg-icon" part="svg">
          <use xlink:href="#svg-logo"></use>
        </svg>`;
      case "wordmark":
        return html` <svg class="svg-icon" part="svg">
          <use xlink:href="#svg-wordmark"></use>
        </svg>`;
      default:
        return `unknown`;
    }
  }

  render() {
    return html`
      <svg
        xmlns="http://www.w3.org/2000/svg"
        xmlns:xlink="http://www.w3.org/1999/xlink"
        style="display:none"
      >
        <symbol id="svg-check" viewBox="0 0 20 20">
          <path
            d="M8.294 16.998c-.435 0-.847-.203-1.11-.553l-3.574-4.72c-.465-.614-.344-1.487.27-1.952.615-.467 1.488-.344 1.953.27l2.35 3.104 5.912-9.492c.407-.652 1.267-.852 1.92-.445.654.406.855 1.266.447 1.92L9.478 16.34c-.242.39-.66.635-1.12.656-.022.002-.042.002-.064.002z"
          />
        </symbol>
        <symbol id="svg-creative-commons-noncommercial-us" viewBox="0 0 20 20">
          <path
            d="M9.988.4c2.69 0 4.966.928 6.825 2.784C18.67 5.04 19.6 7.312 19.6 10s-.913 4.936-2.74 6.744C14.923 18.648 12.63 19.6 9.99 19.6c-2.61 0-4.862-.944-6.753-2.832C1.345 14.88.4 12.624.4 10s.945-4.896 2.835-6.816C5.078 1.328 7.33.4 9.988.4zM2.56 7.42c-.287.81-.43 1.67-.43 2.58 0 2.128.777 3.968 2.33 5.52 1.555 1.552 3.405 2.328 5.552 2.328s4.013-.784 5.6-2.352c.53-.513.967-1.073 1.31-1.68l-3.618-1.61c-.246 1.216-1.33 2.04-2.643 2.136v1.48h-1.1v-1.48c-1.078-.013-2.12-.453-2.915-1.15l1.322-1.333c.637.598 1.274.868 2.143.868.563 0 1.188-.22 1.188-.955 0-.26-.1-.44-.26-.577l-.915-.407-1.14-.508c-.563-.252-1.04-.464-1.52-.677L2.56 7.42zm7.452-5.292c-2.18 0-4.02.768-5.527 2.304-.41.414-.766.846-1.07 1.297l3.67 1.632c.332-1.017 1.3-1.635 2.474-1.704v-1.48h1.1v1.48c.76.037 1.593.245 2.413.88l-1.26 1.297c-.466-.33-1.054-.563-1.642-.563-.476 0-1.15.148-1.15.747 0 .09.03.17.086.242l1.228.547.83.37c.532.236 1.04.46 1.542.685l4.92 2.19c.162-.644.244-1.33.244-2.055 0-2.192-.77-4.048-2.307-5.568-1.522-1.536-3.372-2.304-5.55-2.304z"
          />
        </symbol>
        <symbol id="svg-back-in-time" viewBox="0 0 20 20">
          <path
            d="M11 1.8c-4.445 0-8.06 3.56-8.17 7.995V10H.46l3.593 3.894L7.547 10H4.875v-.205C4.982 6.492 7.683 3.85 11 3.85c3.386 0 6.13 2.754 6.13 6.15 0 3.396-2.744 6.15-6.13 6.15-1.357 0-2.61-.445-3.627-1.193L5.967 16.46C7.355 17.55 9.102 18.2 11 18.2c4.515 0 8.174-3.67 8.174-8.2S15.514 1.8 11 1.8zM10 5v5c0 .13.027.26.077.382s.124.233.216.325l3.2 3.2c.283-.183.55-.39.787-.628L12 11V5h-2z"
          />
        </symbol>
        <symbol id="svg-time-slot" viewBox="0 0 20 20">
          <path
            d="M10 .4C4.698.4.4 4.698.4 10s4.298 9.6 9.6 9.6c5.3 0 9.6-4.298 9.6-9.6S15.3.4 10 .4zm0 17.2c-4.197 0-7.6-3.403-7.6-7.6C2.4 5.8 5.802 2.4 10 2.4V10l6.792-3.396c.513 1.023.808 2.173.808 3.396 0 4.197-3.403 7.6-7.6 7.6z"
          />
        </symbol>
        <symbol id="svg-merge" viewBox="0 0 20 20">
          <path
            d="M17.89 17.707L16.892 20c-3.137-1.366-5.496-3.152-6.892-5.275-1.396 2.123-3.755 3.91-6.892 5.275l-.998-2.293C5.14 16.39 8.55 14.102 8.55 10V7H5.5L10 0l4.5 7h-3.05v3c0 4.102 3.41 6.39 6.44 7.707z"
          />
        </symbol>
        <symbol id="svg-text" viewBox="0 0 20 20">
          <path
            fill-rule="evenodd"
            clip-rule="evenodd"
            d="M15.5 11h-11c-.275 0-.5.225-.5.5v1c0 .276.225.5.5.5h11c.276 0 .5-.224.5-.5v-1c0-.275-.224-.5-.5-.5zm0-4h-11c-.275 0-.5.225-.5.5v1c0 .276.225.5.5.5h11c.276 0 .5-.224.5-.5v-1c0-.275-.224-.5-.5-.5zm-5 8h-6c-.275 0-.5.225-.5.5v1c0 .276.225.5.5.5h6c.276 0 .5-.224.5-.5v-1c0-.275-.224-.5-.5-.5zm5-12h-11c-.275 0-.5.225-.5.5v1c0 .276.225.5.5.5h11c.276 0 .5-.224.5-.5v-1c0-.275-.224-.5-.5-.5z"
          />
        </symbol>
        <symbol id="svg-tv" viewBox="0 0 20 20">
          <path
            d="M18 1H2C.9 1 0 1.9 0 3v11c0 1.1.882 2.178 1.96 2.393l4.373.875S2.57 19 5 19h10c2.43 0-1.334-1.732-1.334-1.732l4.373-.875C19.116 16.178 20 15.1 20 14V3c0-1.1-.9-2-2-2zm0 13H2V3h16v11z"
          />
        </symbol>
        <symbol id="svg-block" viewBox="0 0 20 20">
          <path
            d="M10 .4C4.697.4.4 4.698.4 10c0 5.303 4.297 9.6 9.6 9.6 5.3 0 9.6-4.297 9.6-9.6 0-5.302-4.3-9.6-9.6-9.6zM2.4 10c0-4.197 3.4-7.6 7.6-7.6 1.828 0 3.505.647 4.816 1.723L4.122 14.817C3.046 13.505 2.4 11.83 2.4 10zm7.6 7.6c-1.83 0-3.506-.647-4.816-1.723L15.878 5.184C16.953 6.496 17.6 8.17 17.6 10c0 4.197-3.404 7.6-7.6 7.6z"
          />
        </symbol>
        <symbol id="svg-list" viewBox="0 0 20 20">
          <path
            d="M14.4 9H8.6c-.552 0-.6.447-.6 1s.048 1 .6 1h5.8c.552 0 .6-.447.6-1s-.048-1-.6-1zm2 5H8.6c-.552 0-.6.447-.6 1s.048 1 .6 1h7.8c.552 0 .6-.447.6-1s-.048-1-.6-1zM8.6 6h7.8c.552 0 .6-.447.6-1s-.048-1-.6-1H8.6c-.552 0-.6.447-.6 1s.048 1 .6 1zM5.4 9H3.6c-.552 0-.6.447-.6 1s.048 1 .6 1h1.8c.552 0 .6-.447.6-1s-.048-1-.6-1zm0 5H3.6c-.552 0-.6.447-.6 1s.048 1 .6 1h1.8c.552 0 .6-.447.6-1s-.048-1-.6-1zm0-10H3.6c-.552 0-.6.447-.6 1s.048 1 .6 1h1.8c.552 0 .6-.447.6-1s-.048-1-.6-1z"
          />
        </symbol>
        <symbol id="svg-logo" viewBox="0 0 140 204.1">
          <path
            d="M63.5.3L1.7 31.2c-1 .5-1.7 1.5-1.7 2.7v136.3c0 1.1.6 2.2 1.7 2.7l61.8 30.9c2 1 4.3-.5 4.3-2.7V3c0-2.2-2.3-3.7-4.3-2.7zM76.5 203.8l61.8-30.9c1-.5 1.7-1.5 1.7-2.7v-66.3c0-1.1-.6-2.2-1.7-2.7L76.5 70.3c-2-1-4.3.5-4.3 2.7v128.1c0 2.2 2.3 3.7 4.3 2.7z"
          />
        </symbol>
        <symbol id="svg-wordmark" viewBox="0 0 536.3 106.8">
          <path
            d="M33 .2L.9 16.2c-.6.3-.9.8-.9 1.4v70.8c0 .6.3 1.1.9 1.4l32.1 16c1 .5 2.3-.2 2.3-1.4V1.6C35.2.4 34-.4 33 .2zM39.7 105.8l32.1-16c.5-.3.9-.8.9-1.4V54c0-.6-.3-1.1-.9-1.4l-32.1-16c-1-.5-2.3.2-2.3 1.4v66.5c.1 1.1 1.3 1.8 2.3 1.3zM129.7 34.8c10.8 0 16.6 4 16.6 14.1 0 6.6-2.1 9.8-6.4 12.2 4.7 1.8 7.8 5.2 7.8 12.6 0 11.1-6.7 15.4-17.3 15.4H109V34.8h20.7zm-11.8 7.6V58h11.7c5.4 0 7.8-2.7 7.8-8 0-5.2-2.7-7.5-8.1-7.5h-11.4zm0 23v16.1h12c5.5 0 8.7-1.7 8.7-8.3 0-6.2-4.6-7.9-8.9-7.9h-11.8zM156.6 49.5h8.6v4.8s6.7-4.4 13.5-5.6v8.6c-7.2 1.4-13.4 6.3-13.4 6.3v25.6h-8.6V49.5zM365.4 49.5h8.6v4.8s6.7-4.4 13.5-5.6v8.6c-7.2 1.4-13.4 6.3-13.4 6.3v25.6h-8.6V49.5zM218.4 69.1c0 13.2-4 20.9-17.7 20.9-13.6 0-17.7-7.8-17.7-20.9 0-12.9 4.4-20.5 17.7-20.5s17.7 7.6 17.7 20.5zm-8.7 0c0-9.2-2-13.2-9-13.2s-9 4-9 13.2 1.6 13.6 9 13.6 9-4.4 9-13.6zM232.3 49.5l6.3 32.3h1.6l7.5-31.5h8.9l7.5 31.5h1.6l6.2-32.3h8.6l-8.4 39.7h-13.7L252.2 62 246 89.2h-13.7l-8.4-39.7h8.4zM315.4 57.7s-9.4-1.3-14.1-1.3c-4.8 0-6.9 1.1-6.9 4.4 0 2.6 1.7 3.3 9.4 4.7 9.5 1.7 12.9 4 12.9 12 0 9.3-5.9 12.6-15.7 12.6-5.5 0-14.7-1.7-14.7-1.7l.3-7.2s9.5 1.3 13.6 1.3c5.7 0 7.9-1.2 7.9-4.7 0-2.8-1.3-3.6-9.2-4.9-8.7-1.4-13.2-3.3-13.2-11.7 0-9 7-12.3 14.8-12.3 5.8 0 14.9 1.7 14.9 1.7v7.1zM355.6 81.8l.2 6.4s-9 1.8-16 1.8c-11.9 0-16.5-6.3-16.5-20.3 0-14.5 6.3-21.1 17.2-21.1 11.1 0 16.7 5.8 16.7 18.2l-.6 6.2H332c.1 6.3 2.5 9.5 9 9.5 6.2 0 14.6-.7 14.6-.7zm-7-15.5c0-7.9-2.4-10.6-8.2-10.6-5.9 0-8.5 2.9-8.6 10.6h16.8zM420.5 54.3S412 53 406.8 53c-4.9 0-9.4 1.3-9.4 6.7 0 4.1 2 5.3 10.6 6.7 10.2 1.7 14 3.5 14 11.1 0 9.3-5.8 12.3-15.3 12.3-4.8 0-13.6-1.4-13.6-1.4l.3-4.2s8.9 1.3 12.9 1.3c6.8 0 10.8-1.6 10.8-7.8 0-4.8-2.4-5.8-11.3-7.1-9.1-1.4-13.3-3.1-13.3-10.8 0-8.6 7.1-11.2 14-11.2 6 0 14 1.3 14 1.3v4.4zM432 49.5L442.5 85h2.9L456 49.5h4.8l-16.9 57.3h-4.8l5.1-17.7h-5.5L427 49.4h5zM468.9 89.2V49.5h4.7v2.9s6.7-3.7 12.9-3.7c10.9 0 13.3 5.1 13.3 19.6v20.9H495V68.5c0-11.7-1.3-15.6-9.2-15.6-6.2 0-12.2 3.3-12.2 3.3V89h-4.7zM536.2 49.7l-.2 4s-6.3-.8-9.3-.8c-9.5 0-12.3 4.2-12.3 15.6 0 12.5 1.9 17.1 12.3 17.1 3 0 9.4-.7 9.4-.7l.2 4s-7.1 1-10.5 1c-12.9 0-16.3-5.7-16.3-21.3 0-14.5 4.6-19.9 16.4-19.9 3.4 0 10.3 1 10.3 1z"
          />
        </symbol>
        <symbol id="svg-github" viewBox="0 0 32 32">
          <path
            clip-rule="evenodd"
            d="M16.003 0C7.17 0 .008 7.162.008 15.997c0 7.067 4.582 13.063 10.94 15.18.8.145 1.052-.33 1.052-.753 0-.38.008-1.442 0-2.777-4.45.967-5.37-2.107-5.37-2.107-.728-1.848-1.776-2.34-1.776-2.34-1.452-.992.11-.973.11-.973 1.604.113 2.45 1.65 2.45 1.65 1.427 2.442 3.743 1.736 4.654 1.328.146-1.034.56-1.74 1.017-2.14C9.533 22.663 5.8 21.29 5.8 15.16c0-1.747.622-3.174 1.645-4.292-.165-.404-.715-2.03.157-4.234 0 0 1.343-.43 4.398 1.64 1.276-.354 2.645-.53 4.005-.537 1.36.006 2.727.183 4.005.538 3.055-2.07 4.396-1.64 4.396-1.64.872 2.202.323 3.83.16 4.233 1.022 1.118 1.643 2.545 1.643 4.292 0 6.146-3.74 7.498-7.305 7.893C19.48 23.548 20 24.508 20 26v4.428c0 .428.258.9 1.07.746C27.422 29.054 32 23.062 32 15.997 32 7.162 24.838 0 16.003 0z"
            fill-rule="evenodd"
          />
        </symbol>
        <symbol id="svg-twitter" viewBox="0 0 273.4 222.2">
          <path
            d="M273.4 26.3c-10.1 4.5-20.9 7.5-32.2 8.8 11.6-6.9 20.5-17.9 24.7-31-10.9 6.4-22.9 11.1-35.7 13.6C220 6.8 205.4 0 189.3 0c-31 0-56.1 25.1-56.1 56.1 0 4.4.5 8.7 1.5 12.8C88 66.5 46.7 44.2 19 10.3c-4.8 8.3-7.6 17.9-7.6 28.2 0 19.5 9.9 36.6 25 46.7-9.2-.3-17.8-2.8-25.4-7v.7c0 27.2 19.3 49.8 45 55-4.7 1.3-9.7 2-14.8 2-3.6 0-7.1-.4-10.6-1 7.1 22.3 27.9 38.5 52.4 39-19.2 15-43.4 24-69.7 24-4.5 0-9-.3-13.4-.8 24.8 15.9 54.3 25.2 86 25.2 103.2 0 159.6-85.5 159.6-159.6 0-2.4-.1-4.9-.2-7.3 11.1-8 20.6-17.9 28.1-29.1z"
          />
        </symbol>
        <symbol id="svg-circle-play" viewBox="0 0 191.4 191.4">
          <circle
            fill="none"
            stroke="#FFF"
            stroke-width="22"
            stroke-miterlimit="10"
            cx="95.7"
            cy="95.7"
            r="84.7"
          />
          <path
            d="M87.8 57l46.7 32.6c4.2 3 4.2 9.2 0 12.2l-45.3 31.6c-4.7 3.3-11.1-.1-11.1-5.8V62c0-4.9 5.6-7.9 9.7-5z"
          />
        </symbol>
        <symbol id="svg-code" viewBox="0 0 20 20">
          <path
            d="M5.72 14.75c-.237 0-.475-.083-.665-.252L-.005 10l5.34-4.748c.413-.365 1.045-.33 1.412.083.367.413.33 1.045-.083 1.412L3.004 10l3.38 3.002c.412.367.45 1 .082 1.412-.197.223-.472.336-.747.336zm8.944-.002L20.004 10l-5.06-4.498c-.412-.367-1.044-.33-1.41.083-.367.413-.33 1.045.083 1.412L16.995 10l-3.66 3.252c-.412.367-.45 1-.082 1.412.197.223.472.336.747.336.236 0 .474-.083.664-.252zm-4.678 1.417l2-12c.09-.545-.277-1.06-.822-1.15-.547-.093-1.06.276-1.15.82l-2 12c-.09.546.277 1.06.822 1.152.056.01.11.013.165.013.48 0 .905-.347.986-.835z"
          />
        </symbol>
        <symbol id="svg-menu" viewBox="0 0 20 20">
          <path
            d="M16.4 9H3.6c-.552 0-.6.447-.6 1 0 .553.048 1 .6 1h12.8c.552 0 .6-.447.6-1 0-.553-.048-1-.6-1zm0 4H3.6c-.552 0-.6.447-.6 1 0 .553.048 1 .6 1h12.8c.552 0 .6-.447.6-1 0-.553-.048-1-.6-1zM3.6 7h12.8c.552 0 .6-.447.6-1 0-.553-.048-1-.6-1H3.6c-.552 0-.6.447-.6 1 0 .553.048 1 .6 1z"
          />
        </symbol>
        <symbol id="svg-cross" viewBox="0 0 20 20">
          <path
            d="M14.348 14.85c-.47.468-1.23.468-1.697 0L10 11.82l-2.65 3.028c-.47.47-1.23.47-1.698 0-.47-.47-.47-1.23 0-1.697L8.41 10 5.65 6.85c-.468-.47-.468-1.23 0-1.698.47-.47 1.23-.47 1.698 0L10 8.182l2.65-3.03c.47-.47 1.23-.47 1.698 0 .47.47.47 1.23 0 1.697L11.59 10l2.758 3.15c.47.47.47 1.23 0 1.7z"
          />
        </symbol>
        <symbol id="svg-typeface-reg" viewBox="0 0 113.8 77.2">
          <path
            d="M20.9 0h18.5l20.9 76.1h-8.4l-5.5-19.6H13.9L8.4 76.1H0L20.9 0zm-5.2 49h28.8L33 7.3h-5.7L15.7 49zM107.5 65.9c.2 3.2 2.9 4.4 6.4 4.8l-.3 6.5c-5.8 0-9.8-1.1-13.1-4.4 0 0-9.9 4.4-19.8 4.4-10 0-15.5-5.7-15.5-16.8 0-10.6 5.5-15.2 16.8-16.3l17.3-1.6v-4.7c0-7.7-3.3-10.5-9.9-10.5-7.7 0-20.8 1.4-20.8 1.4l-.3-6.3S80.4 20 89.9 20c12.4 0 17.7 5.7 17.7 17.7v28.2zM82.9 50.3c-6.7.7-9.4 3.9-9.4 9.9 0 6.4 2.8 10.1 8.4 10.1 8.1 0 17.3-3.4 17.3-3.4V48.7l-16.3 1.6z"
          />
        </symbol>
        <symbol id="svg-typeface-bold" viewBox="0 0 114.3 76.6">
          <path
            d="M18.6 0h24.3l18.7 75.4H49.3l-4.1-16.2H16.3l-4.1 16.2H0L18.6 0zm.1 48.4h24.1l-9.2-38.2H28l-9.3 38.2zM109.5 62.4c.2 3.3 1.7 4.6 4.8 5.1l-.3 9.1c-6.7 0-10.6-.9-14.6-4.1 0 0-8.8 4.1-17.7 4.1-10.9 0-16.4-6-16.4-17.5 0-11.7 6.4-15.6 18.1-16.6l14.2-1.2v-4c0-6.1-2.6-7.9-8-7.9-7.4 0-20.7 1.1-20.7 1.1l-.5-8.5s12-2.9 22.1-2.9c13.4 0 18.9 5.6 18.9 18.2v25.1zM84.8 50.9c-5.1.4-7.6 2.9-7.6 7.8s2.1 8 6.7 8c6.3 0 13.6-2.4 13.6-2.4V49.7l-12.7 1.2z"
          />
        </symbol>
        <symbol id="svg-typeface-thin" viewBox="0 0 113.3 78">
          <path
            d="M23.6 0h11.7L59 77h-4l-7.2-23.6H11.1L4 77H0L23.6 0zM12.3 49.6h34.3l-14-45.9h-6.2L12.3 49.6zM105 69.9c.3 3.2 4.4 4.3 8.2 4.6l-.2 3.4c-4.7 0-8.9-1.2-11.3-4.6 0 0-11.2 4.7-22.2 4.7-9 0-14.4-5.4-14.4-16.1 0-9.3 4.4-14.7 15.2-15.8l20.9-2.2v-5.7c0-9.6-4.2-13.4-12.2-13.4s-20.8 1.9-20.8 1.9l-.3-3.7s12.3-2 21.1-2c11.1 0 16.1 5.8 16.1 17.2v31.7zM80.7 49.5c-8.6.9-11.5 4.8-11.5 12.4 0 8 3.7 12.5 10.5 12.5 10.3 0 21.6-4.5 21.6-4.5V47.4l-20.6 2.1z"
          />
        </symbol>
      </svg>
      ${this.icon}
    `;
  }
}

customElements.define("bs-icon", BsIcon);
