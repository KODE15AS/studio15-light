import { defineConfig } from 'vite'
import { svelte, vitePreprocess } from '@sveltejs/vite-plugin-svelte'

export default defineConfig({
  plugins: [svelte({ preprocess: vitePreprocess() })],
  server: {
    // Kun for lokal utvikling mot mock-driveren (WORKSPACE_DRIVER=mock)
    proxy: { '/api': 'http://localhost:8200', '/healthz': 'http://localhost:8200' }
  }
})
