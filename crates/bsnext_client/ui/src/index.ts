import "../styles/style.css";
import "./components/bs-debug";
import {ServerDesc} from "../../generated/dto";
import {html, render} from "lit";

fetch('/__bs_api/servers').then(x => x.json())
  .then((x: ServerDesc) => {
    let next = html`<bs-debug .data=${x}></bs-debug>`
    let app = document.querySelector('#app') as HTMLElement;
    if (!app) throw new Error('cannot...');
    render(next, app);
    // for (let route of x.routes) {
    //   switch (route.kind.kind) {
    //     case "Html":
    //       break;
    //     case "Json":
    //       break;
    //     case "Raw":
    //       break;
    //     case "Sse":
    //       break;
    //     case "Proxy":
    //       break;
    //     case "Dir":
    //       break;
    //
    //   }
    // }
  })
  .catch(console.error)