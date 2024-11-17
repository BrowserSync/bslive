import "../styles/style.css";
import "./components/bs-debug";
import "./components/bs-server-list";
import "./components/bs-server-detail";
import "./components/bs-server-identity";
import "./components/bs-header";
import "./components/bs-icon";
import {GetServersMessageResponseDTO, ServerDesc} from "@browsersync/generated/dto";
import {html, render} from "lit";

const all = fetch('/__bs_api/servers').then(x => x.json())
const me = fetch('/__bs_api/me').then(x => x.json())

Promise.all([all, me])
  .then(([servers, me]: [GetServersMessageResponseDTO, ServerDesc]) => {
    let next = html`
        <bs-debug .servers=${servers} .me=${me}></bs-debug>`
    let app = document.querySelector('#app') as HTMLElement;
    if (!app) throw new Error('cannot...');
    // console.log(x);
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