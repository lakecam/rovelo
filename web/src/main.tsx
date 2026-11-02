import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import App from "./App";
import { loadCore } from "./wasm";
import { initStore } from "./state/store";

// Rust core first (diagram logic lives in WebAssembly), then the UI.
loadCore().then(() => {
  initStore();
  createRoot(document.getElementById("root")!).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
});
