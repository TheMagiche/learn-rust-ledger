/// <reference types="vitest/config" />
import tailwindcss from "@tailwindcss/vite"
import react from "@vitejs/plugin-react"
import { defineConfig } from "vitest/config"

const port = Number(process.env.WEB_PORT ?? 5173)

function siteBase(): string {
  const raw = process.env.VITE_BASE?.trim()
  if (!raw || raw === "/") {
    return "/"
  }
  const prefixed = raw.startsWith("/") ? raw : `/${raw}`
  return prefixed.endsWith("/") ? prefixed : `${prefixed}/`
}

export default defineConfig({
  base: siteBase(),
  plugins: [react(), tailwindcss()],
  server: {
    host: "127.0.0.1",
    port,
    strictPort: true,
    proxy: {
      "/api": "http://127.0.0.1:8080",
    },
  },
  test: {
    environment: "jsdom",
    css: false,
  },
})
