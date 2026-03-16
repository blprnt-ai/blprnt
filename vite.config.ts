import { platform } from 'node:os'
import { resolve } from 'node:path'
import { sentryVitePlugin } from '@sentry/vite-plugin'
import tailwindcss from '@tailwindcss/vite'
import react from '@vitejs/plugin-react'
import observerPlugin from 'mobx-react-observer/babel-plugin'
import { defineConfig, loadEnv } from 'vite'

const isWindows = platform() === 'win32'

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '')

  const sentryPlugin = [
    env.CI || mode === 'development'
      ? []
      : sentryVitePlugin({
          authToken: env.SENTRY_AUTH_TOKEN,
          org: 'blprntai',
          project: 'rust',
          telemetry: false,
        }),
  ]

  return {
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
      sourcemap: !env.CI,
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
          : (path: string) => {
              const rootPath = resolve(__dirname, './')
              const srcPath = resolve(__dirname, './src')
              return !(path === rootPath || path.includes(srcPath))
            },
        interval: 100,
        usePolling: isWindows,
      },
    },
  }
})
