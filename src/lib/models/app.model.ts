import { flow, makeAutoObservable, onBecomeObserved } from 'mobx'
import type { LlmModelResponse, PersonalityModelDto, SkillItem } from '@/bindings'
import { AppApi } from '@/lib/api/app.api'
import { tauriPersonalitiesApi } from '@/lib/api/tauri/personalities.api'
import { tauriProvidersApi } from '@/lib/api/tauri/providers.api'

export enum AppState {
  Loading = 'loading',
  FirstLoad = 'first-load',
  Ready = 'ready',
}

export interface ModelCatalogItem extends LlmModelResponse {
  toggledOn: boolean
  provider: string
}

export interface AppSnapshot {
  state: AppState
  isWindowFocused: boolean
  modelsCatalog: ModelCatalogItem[]
  skills: SkillItem[]
}

export class AppModel {
  public state: AppState = AppState.Loading
  public isWindowFocused = true
  public modelsCatalog: ModelCatalogItem[] = []
  public personalities: PersonalityModelDto[] = []
  public skills: SkillItem[] = []
  readonly api = new AppApi()

  constructor(snapshot?: Partial<AppSnapshot>) {
    Object.assign(this, snapshot)
    makeAutoObservable(this, { api: false }, { autoBind: true })

    onBecomeObserved(this, 'modelsCatalog', this.loadModelsCatalog)
    onBecomeObserved(this, 'personalities', this.loadPersonalities)
    onBecomeObserved(this, 'skills', this.listSkills)
  }

  setState = (state: AppState) => {
    this.state = state
  }

  setWindowFocused = (focused: boolean) => {
    this.isWindowFocused = focused
  }

  setModelsCatalog = (modelsCatalog: ModelCatalogItem[]) => {
    this.modelsCatalog = modelsCatalog
  }

  loadModelsCatalog = flow(function* (this: AppModel) {
    const result: LlmModelResponse[] = yield tauriProvidersApi.getModels()

    const catalog = result.map((model) => {
      const stripFreeName = model.name.replace('(free)', '').trim()
      const nameParts = stripFreeName.split(':')
      const name = nameParts.length === 1 ? nameParts[0] : nameParts[1]
      const provider = model.slug.split('/')[0].trim()

      return { ...model, name, provider, toggledOn: false }
    })
    this.modelsCatalog = catalog
    return catalog
  })

  loadPersonalities = flow(function* (this: AppModel) {
    const personalities: PersonalityModelDto[] = yield tauriPersonalitiesApi.list()
    this.personalities = personalities
    return personalities
  })

  listSkills = flow(function* (this: AppModel) {
    const skills: SkillItem[] = yield this.api.listSkills()
    this.skills = skills

    return skills
  })

  buildHash = () => this.api.buildHash()
  openDevtools = () => this.api.openDevtools()
  reloadWindow = () => this.api.reloadWindow()
  frontendReady = () => this.api.frontendReady()
}
