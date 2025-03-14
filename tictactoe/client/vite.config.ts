import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

export default defineConfig({
  plugins: [vue()],
  build: {
    outDir: 'dist'
  },
  server: {
    proxy: {
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true
      },
      '/games': {
        target: 'http://localhost:8000',
        changeOrigin: true
      },
      '/identities': {
        target: 'http://localhost:8000',
        changeOrigin: true
      }
    }
  }
});
