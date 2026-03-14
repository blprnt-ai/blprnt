import { load, type Store } from '@tauri-apps/plugin-store'
import { flow, makeAutoObservable } from 'mobx'
import { useEffect, useRef } from 'react'

const FORCE_SHOW_TOUR = false
const FORCE_SHOW_INTRO_SCREEN = false

class BlprntConfig {
  private store: Store | null = null

  seenIntroScreen = false
  seenTour = false
  useExplicitDelete = true

  constructor() {
    makeAutoObservable<BlprntConfig, 'store'>(this, { store: false }, { autoBind: true })
    this.load()
  }

  public load = flow(function* (this: BlprntConfig) {
    this.store = yield load('blprnt-config', {
      defaults: {
        seenIntroScreen: false,
        seenTour: false,
        useExplicitDelete: true,
      },
    })

    this.seenIntroScreen = (yield this.store?.get('seenIntroScreen')) ?? false
    this.seenTour = (yield this.store?.get('seenTour')) ?? false
    this.useExplicitDelete = (yield this.store?.get('useExplicitDelete')) ?? true

    if (FORCE_SHOW_INTRO_SCREEN) this.seenIntroScreen = false
    if (FORCE_SHOW_TOUR) this.seenTour = false
  })

  public setSeenIntroScreen(seenIntroScreen: boolean) {
    this.seenIntroScreen = seenIntroScreen
    this.store?.set('seenIntroScreen', seenIntroScreen)
  }

  public setSeenTour(seenTour: boolean) {
    this.seenTour = seenTour
    this.store?.set('seenTour', seenTour)
  }

  public setUseExplicitDelete(useExplicitDelete: boolean) {
    this.useExplicitDelete = useExplicitDelete
    this.store?.set('useExplicitDelete', useExplicitDelete)
  }
}

export const blprntConfig = new BlprntConfig()

export const useBlprntConfig = () => {
  const config = useRef(blprntConfig)

  useEffect(() => {
    config.current.load()
  }, [])

  return config.current
}
