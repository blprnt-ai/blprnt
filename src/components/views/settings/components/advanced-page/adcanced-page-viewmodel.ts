import { load, type Store } from '@tauri-apps/plugin-store'
import { flow, makeAutoObservable } from 'mobx'
import { createContext, useContext } from 'react'
import type { JsRuntimeHealthStatus, JsRuntimeInstallResult } from '@/bindings'
import { basicToast } from '@/components/atoms/toaster'
// eslint-disable-next-line
import { tauriCommandApi } from '@/lib/api/tauri/command.api'
import {
  ADVANCED_REASONING_EFFORT_CLASSIFIER_ENABLED_KEY,
  ADVANCED_SKILL_MATCHER_ENABLED_KEY,
  storeBoolWithDefaultTrue,
} from '@/lib/utils/blprnt-settings'

const storeFile = 'blprnt.json'

export class AdvancedPageViewModel {
  public store: Store | null = null
  public settingsLoaded = false

  public reasoningEffortClassifierEnabled = true
  public skillMatcherEnabled = true

  public jsRuntimeHealth: JsRuntimeHealthStatus | null = null
  public jsRuntimeLoading = false
  public jsRuntimeInstalling = false

  constructor() {
    makeAutoObservable(this, { store: false }, { autoBind: true })
  }

  public loadSettings = flow(function* (this: AdvancedPageViewModel) {
    const store = yield load(storeFile)
    const reasoningEffortClassifierEnabled = yield store.get(ADVANCED_REASONING_EFFORT_CLASSIFIER_ENABLED_KEY)
    const skillMatcherEnabled = yield store.get(ADVANCED_SKILL_MATCHER_ENABLED_KEY)

    this.store = store
    this.reasoningEffortClassifierEnabled = storeBoolWithDefaultTrue(reasoningEffortClassifierEnabled)
    this.skillMatcherEnabled = storeBoolWithDefaultTrue(skillMatcherEnabled)
    this.settingsLoaded = true
  })

  public loadJsRuntimeHealth = flow(function* (this: AdvancedPageViewModel) {
    this.jsRuntimeLoading = true
    try {
      this.jsRuntimeHealth = yield tauriCommandApi.jsRuntimeHealthStatus()
    } finally {
      this.jsRuntimeLoading = false
    }
  })

  public installManagedJsRuntime = flow(function* (this: AdvancedPageViewModel, overwrite: boolean) {
    const toastId = 'js-runtime-install'
    this.jsRuntimeInstalling = true
    basicToast.loading({ id: toastId, title: 'Installing runtime...' })
    try {
      const result: JsRuntimeInstallResult = yield tauriCommandApi.jsRuntimeInstallManaged(overwrite)
      this.jsRuntimeHealth = result.status
      basicToast.success({ id: toastId, title: 'Runtime installed' })
    } catch (error: unknown) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error'
      basicToast.error({ description: errorMessage, id: toastId, title: 'Failed to install runtime' })
    } finally {
      this.jsRuntimeInstalling = false
    }
  })

  public setReasoningEffortClassifierEnabled(enabled: boolean) {
    this.reasoningEffortClassifierEnabled = enabled
  }

  public setSkillMatcherEnabled(enabled: boolean) {
    this.skillMatcherEnabled = enabled
  }

  public persistSettings = flow(function* (
    this: AdvancedPageViewModel,
    settings: Partial<Pick<AdvancedPageViewModel, 'reasoningEffortClassifierEnabled' | 'skillMatcherEnabled'>>,
  ) {
    if (!this.store || !this.settingsLoaded) return

    if (settings.reasoningEffortClassifierEnabled !== undefined) {
      this.reasoningEffortClassifierEnabled = settings.reasoningEffortClassifierEnabled
      yield this.store.set(ADVANCED_REASONING_EFFORT_CLASSIFIER_ENABLED_KEY, settings.reasoningEffortClassifierEnabled)
    }

    if (settings.skillMatcherEnabled !== undefined) {
      this.skillMatcherEnabled = settings.skillMatcherEnabled
      yield this.store.set(ADVANCED_SKILL_MATCHER_ENABLED_KEY, settings.skillMatcherEnabled)
    }

    yield this.store.save()
  })
}

export const AdvancedPageViewmodelContext = createContext<AdvancedPageViewModel>(new AdvancedPageViewModel())
export const useAdvancedPageViewModel = () => {
  const viewmodel = useContext(AdvancedPageViewmodelContext)
  if (!viewmodel) throw new Error('useAdvancedPageViewModel must be used within AdvancedPageViewmodelContext')
  return viewmodel
}
