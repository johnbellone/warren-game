import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  base: './',
  server: {
    port: 1420,
    host: true,
  },
  build: {
    outDir: 'client/dist',
    emptyOutDir: true,
  },
})
