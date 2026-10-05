import { defineConfig } from 'vite'
import { svelte, vitePreprocess } from '@sveltejs/vite-plugin-svelte'

// WEB_BASE settes av arbeidsflate-containeren (f.eks. /web/<flate>/) slik
// at websiden serveres bak den interne proxyen med riktig sti-prefiks.
export default defineConfig({
  base: process.env.WEB_BASE || '/',
  plugins: [svelte({ preprocess: vitePreprocess() })],
  server: {
    host: true,
    port: 5173,
    strictPort: true,
    // Lukket tailnett — verten varierer (ts.net-navn), så vertsjekken er av.
    allowedHosts: true
  }
})
