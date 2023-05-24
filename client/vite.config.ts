import Unocss from 'unocss/vite';
import { defineConfig } from 'vite';
import solidPlugin from 'vite-plugin-solid';

export default defineConfig({
  plugins: [solidPlugin(), Unocss()],
  build: {
    target: 'esnext',
  },
  envDir: '../server',
  envPrefix: 'CLIENT_',
  server: { proxy: { '/api': 'http://localhost:8080/' } },
});
