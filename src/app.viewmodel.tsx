import { StateFlags, saveWindowState } from '@tauri-apps/plugin-window-state'
import { debounce } from 'lodash'
import { flow, makeAutoObservable } from 'mobx'
import { AppModel, AppState } from '@/lib/models/app.model'
import { startEventBusListeners } from './lib/events'
import { BannerModel } from './lib/models/banner.model'
import { ProviderModel } from './lib/models/provider.model'

export class AppViewModel {
  public readonly appModel = new AppModel()
  public providers: ProviderModel[] = []
  public bannerModel = new BannerModel()
  public isSidebarExpanded = true

  constructor() {
    makeAutoObservable<AppViewModel>(
      this,
      {
        appModel: false,
      },
      { autoBind: true },
    )

    void this.init()
  }

  get state() {
    return this.appModel.state
  }

  get isLoading() {
    return this.state === AppState.Loading
  }

  get models() {
    return this.appModel.modelsCatalog
  }

  get personalities() {
    return this.appModel.personalities
  }

  get skills() {
    return this.appModel.skills
  }

  get hasCodex() {
    return this.providers.some((p) => p.provider === 'openai_fnf')
  }

  get hasClaude() {
    return this.providers.some((p) => p.provider === 'anthropic_fnf')
  }

  setIsLoading = () => {
    this.appModel.setState(AppState.Loading)
  }

  setReady = () => {
    this.appModel.setState(AppState.Ready)
  }

  toggleSidebarExpanded = () => {
    this.isSidebarExpanded = !this.isSidebarExpanded
  }

  init = flow(function* (this: AppViewModel) {
    console.log('Starting event listeners')
    yield this.appModel.frontendReady()
    startEventBusListeners()

    yield this.finishAppEntry()
  })

  finishAppEntry = async () => {
    this.listenWindow()
    this.setReady()

    await this.refreshProviders()
  }

  refreshProviders = flow(function* (this: AppViewModel) {
    const list = yield ProviderModel.list()
    this.providers = list
  })

  setWindowFocused = (focused: boolean) => {
    this.appModel.setWindowFocused(focused)
  }

  listenWindow = () => {
    this.setWindowFocused(document.hasFocus() && !document.hidden)
    window.addEventListener('resize', this.handleResize)
    window.addEventListener('focus', this.handleFocus)
    window.addEventListener('blur', this.handleBlur)
    document.addEventListener('visibilitychange', this.handleVisibilityChange)
  }

  unlistenWindow = () => {
    window.removeEventListener('resize', this.handleResize)
    window.removeEventListener('focus', this.handleFocus)
    window.removeEventListener('blur', this.handleBlur)
    document.removeEventListener('visibilitychange', this.handleVisibilityChange)
  }

  handleFocus = () => {
    this.setWindowFocused(true)
  }

  handleBlur = () => {
    this.setWindowFocused(false)
  }

  handleVisibilityChange = () => {
    if (document.hidden) {
      this.setWindowFocused(false)
      return
    }

    this.setWindowFocused(document.hasFocus())
  }

  handleResize = debounce(() => {
    saveWindowState(StateFlags.SIZE | StateFlags.POSITION)
  }, 100)
}
