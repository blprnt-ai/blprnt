import { spawn } from 'node:child_process'

function hasMaxOldSpaceOption(nodeOptions) {
  return /--max-old-space-size=\d+/.test(nodeOptions)
}

const existingNodeOptions = process.env.NODE_OPTIONS?.trim() ?? ''
const defaultMaxOldSpaceMb = process.env.CI ? '3072' : '8192'
const maxOldSpaceMb = process.env.BLPRNT_MAX_OLD_SPACE_SIZE_MB ?? defaultMaxOldSpaceMb

const nodeOptions = hasMaxOldSpaceOption(existingNodeOptions)
  ? existingNodeOptions
  : [existingNodeOptions, `--max-old-space-size=${maxOldSpaceMb}`].filter(Boolean).join(' ')

const viteArgs = process.argv.slice(2)

const child = spawn(process.execPath, ['./node_modules/vite/bin/vite.js', 'build', ...viteArgs], {
  stdio: 'inherit',
  env: {
    ...process.env,
    NODE_OPTIONS: nodeOptions,
  },
})

child.on('exit', (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal)
    return
  }

  process.exit(code ?? 1)
})
