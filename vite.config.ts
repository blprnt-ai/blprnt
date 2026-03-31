import { resolve } from 'node:path'
import tailwindcss from '@tailwindcss/vite'
import { tanstackRouter } from '@tanstack/router-plugin/vite'
import react from '@vitejs/plugin-react'
import observerPlugin from 'mobx-react-observer/babel-plugin'
import { defineConfig, loadEnv } from 'vite'

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '')

  return {
    build: {
      outDir: './dist',
      rollupOptions: {
        input: {
          main: resolve(__dirname, 'index.html'),
        },
      },
      sourcemap: !env.CI,
    },
    plugins: [
      tanstackRouter({
        autoCodeSplitting: true,
        target: 'react',
      }),
      react({
        babel: {
          plugins: [observerPlugin()],
        },
      }),
      tailwindcss(),
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
    },
  }
})
