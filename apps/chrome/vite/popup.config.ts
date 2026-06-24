import babel from '@rolldown/plugin-babel';
import react, { reactCompilerPreset } from '@vitejs/plugin-react';
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
    rolldownOptions: {
      input: {
        popup: 'index.html',
        options: 'options.html',
      },
    },
  },
  plugins: [react(), babel({ presets: [reactCompilerPreset()] })],
});
