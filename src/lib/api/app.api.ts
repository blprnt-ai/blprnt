import { tauriCommandApi } from './tauri/command.api'
import { tauriSessionApi } from './tauri/session.api'

export class AppApi {
  public buildHash = () => tauriCommandApi.buildHash()
  public openDevtools = () => tauriCommandApi.devtools()
  public reloadWindow = () => tauriCommandApi.reload()
  public frontendReady = () => tauriCommandApi.frontendReady()

  public listSkills = () => tauriSessionApi.listSkills()
}
