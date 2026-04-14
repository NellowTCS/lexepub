// vite.config.js
import { defineConfig } from 'vite';
import { VitePWA } from 'vite-plugin-pwa';

export default defineConfig({
  root: '/lexepub/',
  base: './',
  plugins: [
    VitePWA({
      registerType: 'autoUpdate',
      includeAssets: ['robots.txt'],
      manifest: {
        name: 'HTMLReader',
        short_name: 'HTMLReader',
        start_url: '/lexepub/',
        display: 'standalone',
        background_color: '#f5f5f5',
        theme_color: '#2196F3',
      },
      workbox: {
        runtimeCaching: [
          {
            urlPattern: /.*\.(js|css|html)$/,
            handler: 'NetworkFirst',
            options: { cacheName: 'app-shell' },
          },
          {
            urlPattern: /.*\.(png|ico|json)$/,
            handler: 'CacheFirst',
            options: { cacheName: 'assets' },
          },
        ],
      },
    }),
  ],
  server: { open: true, allowedHosts: true },
  build: { sourcemap: true, outDir: './dist', emptyOutDir: true },
});
