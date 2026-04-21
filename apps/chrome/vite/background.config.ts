import { defineConfig } from "vite";

export default defineConfig({
  build: {
    emptyOutDir: false,
    target: "node16",
    rolldownOptions: {
      input: {
        background: "src/background/index.tsx",
      },
      output: {
        entryFileNames: "[name].js",
      },
    },
  },
});
