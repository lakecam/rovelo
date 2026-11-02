import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "node:path";

// Tauri's dev shell expects a fixed port and no screen clearing.
export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: { alias: { "@": path.resolve(import.meta.dirname, "src") } },
  clearScreen: false,
  // Icons live in /assets/icons, outside web/.
  server: { port: 5173, strictPort: true, fs: { allow: [".."] } },
  build: { target: "es2022" },
  test: { environment: "jsdom", include: ["src/**/*.test.{ts,tsx}"], globals: false },
});
