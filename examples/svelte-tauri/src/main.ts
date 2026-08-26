import { mount } from "svelte";

import App from "./App.svelte";

import "./style.css";

mount(App, { target: document.querySelector("#app")! });
if (import.meta.env.MODE === "e2e") {
  void import("@wdio/tauri-plugin").catch(console.error);
}
