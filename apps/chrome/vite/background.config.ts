import { URL, fileURLToPath } from 'node:url';
import { defineConfig } from 'vite';

export default defineConfig({
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('../src', import.meta.url)),
    },
  },
  build: {
    emptyOutDir: false,
    target: 'node16',
    rolldownOptions: {
      input: {
        background: 'src/background/index.tsx',
      },
      output: {
        entryFileNames: '[name].js',
      },
    },
  },
});
