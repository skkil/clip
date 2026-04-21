import { defineConfig } from "vite";

export default defineConfig({
  build: {
    emptyOutDir: false,
    rolldownOptions: {
      input: {
        content: "src/content/index.tsx",
      },
      output: {
        entryFileNames: "[name].js",
      },
    },
  },
});
