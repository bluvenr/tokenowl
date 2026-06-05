import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App";
import { TrayPopup } from "./components/tray/TrayPopup";
import { ErrorBoundary } from "./components/ErrorBoundary";
import "./styles/globals.css";
import "./locales/i18n";

// Detect window label: "tray" renders the compact popup, "main" renders the full app
const windowLabel = getCurrentWindow().label;

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ErrorBoundary>
      {windowLabel === "tray" ? <TrayPopup /> : <App />}
    </ErrorBoundary>
  </React.StrictMode>,
);
