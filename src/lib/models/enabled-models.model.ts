import { load, type Store } from '@tauri-apps/plugin-store'
import { makeAutoObservable, observable, runInAction } from 'mobx'

const storeFile = '.models.store'
const storeKey = 'enabled-model-slugs'

export class EnabledModelsModel {
  public store: Store | null = null
  public isLoaded = false
  public enabledSlugs = observable.set<string>([])

  constructor() {
    makeAutoObservable(this, { store: false }, { autoBind: true })
    this.load()
  }

  get slugs() {
    return Array.from(this.enabledSlugs)
  }

  hasSlug = (slug: string) => this.enabledSlugs.has(slug)

  private persist = () => {
    if (!this.store) return
    this.store.set(storeKey, this.slugs)
  }

  load = async () => {
    const store = await load(storeFile)
    const slugs = (await store.get<string[]>(storeKey)) ?? []

    runInAction(() => {
      this.store = store
      this.enabledSlugs.clear()
      for (const slug of slugs) this.enabledSlugs.add(slug)
      this.isLoaded = true
    })
  }

  addSlug = (slug: string) => {
    if (!this.isLoaded || this.enabledSlugs.has(slug)) return
    this.enabledSlugs.add(slug)
    this.persist()
  }

  removeSlug = (slug: string) => {
    if (!this.isLoaded || !this.enabledSlugs.has(slug)) return
    this.enabledSlugs.delete(slug)
    this.persist()
  }

  toggleSlug = (slug: string) => {
    if (this.enabledSlugs.has(slug)) this.removeSlug(slug)
    else this.addSlug(slug)
  }
}

export const enabledModelsModel = new EnabledModelsModel()
