import type { BunRuntimeInstallResult, BunRuntimeStatus } from '@/bindings'
import { commands } from '@/bindings'

class TauriCommandApi {
  public async frontendReady() {
    const result = await commands.frontendReady()
    if (result.status === 'error') throw result.error
  }

  public async devtools() {
    const result = await commands.openDevtools()
    if (result.status === 'error') throw result.error
  }

  public async reload() {
    const result = await commands.reloadWindow()
    if (result.status === 'error') throw result.error
  }

  public async buildHash() {
    const result = await commands.getBuildHash()
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async bunRuntimeStatus(): Promise<BunRuntimeStatus> {
    const result = await commands.bunRuntimeStatus()
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async bunRuntimeInstallUserLocal(overwrite: boolean): Promise<BunRuntimeInstallResult> {
    const result = await commands.bunRuntimeInstallUserLocal(overwrite)
    if (result.status === 'error') throw result.error

    return result.data
  }
}

export const tauriCommandApi = new TauriCommandApi()
