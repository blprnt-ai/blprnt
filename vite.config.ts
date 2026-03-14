import { platform } from 'node:os'
import { resolve } from 'node:path'
import { sentryVitePlugin } from '@sentry/vite-plugin'
import tailwindcss from '@tailwindcss/vite'
import react from '@vitejs/plugin-react'
import observerPlugin from 'mobx-react-observer/babel-plugin'
import { defineConfig } from 'vite'

const isWindows = platform() === 'win32'

const sentryPlugin = [
  process.env.CI || process.env.NODE_ENV === 'development'
    ? []
    : sentryVitePlugin({
        authToken:
          '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',
        org: 'blprntai',
        project: 'rust',
        telemetry: false,
      }),
]

export default defineConfig({
  build: {
    cssCodeSplit: false,
    outDir: './dist',
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
      },
      output: {
        inlineDynamicImports: true,
      },
    },
    sourcemap: !process.env.CI,
  },
  clearScreen: false,
  plugins: [
    react({
      babel: {
        plugins: [observerPlugin()],
      },
    }),
    tailwindcss(),
    ...sentryPlugin,
  ],
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
    },
  },
  server: {
    hmr: {
      host: 'localhost',
      overlay: true,
      port: 7181,
    },
    host: 'localhost',
    port: 7181,
    strictPort: true,
    watch: {
      depth: 99,
      ignored: isWindows
        ? ['**/node_modules/**', '**/tauri-src/**', '**/.git/**']
        : (path) => {
            const rootPath = resolve(__dirname, './')
            const srcPath = resolve(__dirname, './src')
            return !(path === rootPath || path.includes(srcPath))
          },
      interval: 100,
      usePolling: isWindows,
    },
  },
})
