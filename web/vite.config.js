import { defineConfig } from 'vite';

export default defineConfig({
  server: {
    proxy: {
      '/api/wallet': {
        target: 'http://127.0.0.1:3000',
        changeOrigin: false,
        rewrite: (path) => path.replace(/^\/api/, ''),
      },
    },
  },
});
