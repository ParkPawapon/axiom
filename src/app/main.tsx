import React from "react";
import ReactDOM from "react-dom/client";

import { App } from "./app";
import "./styles/fonts.css";
import "./styles/globals.css";

const rootElement = document.getElementById("root");

if (!rootElement) {
  throw new Error("AxiomPHP root element was not found.");
}

ReactDOM.createRoot(rootElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
