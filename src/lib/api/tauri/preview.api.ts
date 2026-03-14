import { commands, type PreviewStartParams } from '@/bindings'

export type PreviewMode = 'dev' | 'static'
export type PreviewSessionStatus = 'starting' | 'ready' | 'error' | 'stopped'
export type PreviewServerAction = 'attached' | 'started'

class TauriPreviewApi {
  public async start(params: PreviewStartParams) {
    return commands.previewStart(params)
  }

  public async stop(projectId: string) {
    await commands.previewStop(projectId)
  }

  public async reload(projectId: string) {
    return commands.previewReload(projectId)
  }

  public async status(projectId: string) {
    return commands.previewStatus(projectId)
  }
}

export const tauriPreviewApi = new TauriPreviewApi()
