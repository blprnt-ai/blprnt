import type { BunRuntimeInstallResult, BunRuntimeStatus, ReportBugSubmitRequest } from '@/bindings'
import { commands } from '@/bindings'
import type { ReportBugSubmitRequestPayload } from '@/lib/models/report-bug.types'

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

  public async reportBugSubmit(request: ReportBugSubmitRequestPayload) {
    const result = await commands.reportBugSubmit(request as unknown as ReportBugSubmitRequest)
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
