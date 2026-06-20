import { defineConfig } from "vite";
import react, { reactCompilerPreset } from "@vitejs/plugin-react";
import babel from "@rolldown/plugin-babel";

export default defineConfig({
  build: {
    emptyOutDir: false,
    rolldownOptions: {
      input: {
        popup: "index.html",
        options: "options.html",
      },
    },
  },
  plugins: [react(), babel({ presets: [reactCompilerPreset()] })],
});
