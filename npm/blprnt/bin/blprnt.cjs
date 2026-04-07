#!/usr/bin/env node

const { spawn } = require('node:child_process')
const path = require('node:path')

const PACKAGE_BY_PLATFORM = {
  'darwin:arm64': {
    binary: 'blprnt',
    packageName: '@blprnt/blprnt-darwin-arm64',
  },
  'linux:x64': {
    binary: 'blprnt',
    packageName: '@blprnt/blprnt-linux-x64',
  },
  'win32:x64': {
    binary: 'blprnt.exe',
    packageName: '@blprnt/blprnt-win32-x64',
  },
}

function fail(message) {
  console.error(message)
  process.exit(1)
}

function resolveInstalledBinary() {
  const key = `${process.platform}:${process.arch}`
  const platformPackage = PACKAGE_BY_PLATFORM[key]

  if (!platformPackage) {
    fail(`Unsupported platform: ${process.platform} ${process.arch}`)
  }

  let packageJsonPath
  try {
    packageJsonPath = require.resolve(`${platformPackage.packageName}/package.json`)
  } catch (error) {
    fail(`Missing optional dependency: ${platformPackage.packageName}`)
  }

  return path.join(path.dirname(packageJsonPath), platformPackage.binary)
}

function resolveStaticBaseDir() {
  return path.join(__dirname, '..', 'dist')
}

const binaryPath = resolveInstalledBinary()
const env = {
  ...process.env,
  BLPRNT_BASE_DIR: process.env.BLPRNT_BASE_DIR || resolveStaticBaseDir(),
}

const child = spawn(binaryPath, process.argv.slice(2), {
  env,
  stdio: 'inherit',
})

child.on('error', (error) => {
  fail(error instanceof Error ? error.message : String(error))
})

child.on('exit', (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal)
    return
  }

  process.exit(code ?? 1)
})
