import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { readFileSync } from 'fs';
import { join } from 'path';

const pkg = readFileSync(join(__dirname, 'package.json'), 'utf8');
const version = JSON.parse(pkg).version;

export default defineConfig({
  plugins: [react()],
  base: '/app',
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true,
      },
    },
  },
  build: {
    outDir: '../dist',
    emptyOutDir: true,
    assetsDir: 'assets',
    assetsInlineLimit: 10000000,
    chunkSizeWarningLimit: 1024,
  },
  define: {
    __APP_VERSION__: JSON.stringify(version),
  },
});