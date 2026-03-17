import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { JsRuntimeHealthStatus } from '@/bindings'
import { basicToast } from '@/components/atoms/toaster'
import { tauriCommandApi } from '@/lib/api/tauri/command.api'
import { AdvancedPageViewModel } from './adcanced-page-viewmodel'

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
}))

vi.mock('@tauri-apps/plugin-store', () => ({
  load: vi.fn().mockResolvedValue({
    get: vi.fn().mockResolvedValue(undefined),
    save: vi.fn().mockResolvedValue(undefined),
    set: vi.fn().mockResolvedValue(undefined),
  }),
}))

vi.mock('@/components/atoms/toaster', () => ({
  basicToast: {
    error: vi.fn(),
    loading: vi.fn(),
    success: vi.fn(),
  },
}))

vi.mock('@/lib/api/tauri/command.api', () => ({
  tauriCommandApi: {
    jsRuntimeHealthStatus: vi.fn(),
    jsRuntimeInstallManaged: vi.fn(),
  },
}))

const createRuntimeHealth = (overrides: Partial<JsRuntimeHealthStatus> = {}): JsRuntimeHealthStatus =>
  ({
    active_runtime: {
      command: 'bun',
      kind: 'bun',
      source: 'path',
      version: '1.2.0',
    },
    install_supported: true,
    managed_runtime: {
      command: '/Users/supagoku/.local/bin/bun',
      detected_version: '1.2.0',
      error: null,
      state: 'available',
    },
    managed_runtime_path: '/Users/supagoku/.local/bin/bun',
    path_help_snip: 'export PATH="$HOME/.local/bin:$PATH"',
    qmd_readiness: {
      detail: 'QMD is ready',
      state: 'ready',
    },
    recommended_action: {
      detail: 'Runtime is ready',
      type: 'none',
    },
    runtime_on_path: {
      command: 'bun',
      detected_version: '1.2.0',
      error: null,
      state: 'available',
    },
    ...overrides,
  }) as JsRuntimeHealthStatus

describe('AdvancedPageViewModel runtime health', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('loads runtime health on every supported platform', async () => {
    const health = createRuntimeHealth()
    vi.mocked(tauriCommandApi.jsRuntimeHealthStatus).mockResolvedValue(health)

    const viewmodel = new AdvancedPageViewModel()

    await viewmodel.loadJsRuntimeHealth()

    expect(tauriCommandApi.jsRuntimeHealthStatus).toHaveBeenCalledTimes(1)
    expect(viewmodel.jsRuntimeHealth).toEqual(health)
    expect(viewmodel.jsRuntimeLoading).toBe(false)
  })

  it('updates runtime health after a managed install succeeds', async () => {
    const installedHealth = createRuntimeHealth({
      active_runtime: {
        command: '/Users/supagoku/.local/bin/bun',
        kind: 'bun',
        source: 'managed',
        version: '1.2.0',
      },
      recommended_action: {
        detail: 'Add the managed runtime to PATH',
        type: 'add_to_path',
      },
      runtime_on_path: {
        command: 'bun',
        detected_version: null,
        error: null,
        state: 'missing',
      },
    })
    vi.mocked(tauriCommandApi.jsRuntimeInstallManaged).mockResolvedValue({
      path_help_snip: 'export PATH="$HOME/.local/bin:$PATH"',
      status: installedHealth,
    })

    const viewmodel = new AdvancedPageViewModel()

    await viewmodel.installManagedJsRuntime(false)

    expect(tauriCommandApi.jsRuntimeInstallManaged).toHaveBeenCalledWith(false)
    expect(viewmodel.jsRuntimeHealth).toEqual(installedHealth)
    expect(viewmodel.jsRuntimeInstalling).toBe(false)
    expect(basicToast.loading).toHaveBeenCalledWith({ id: 'js-runtime-install', title: 'Installing runtime...' })
    expect(basicToast.success).toHaveBeenCalledWith({ id: 'js-runtime-install', title: 'Runtime installed' })
  })

  it('resets install state and reports failures', async () => {
    vi.mocked(tauriCommandApi.jsRuntimeInstallManaged).mockRejectedValue(new Error('install failed'))

    const viewmodel = new AdvancedPageViewModel()

    await viewmodel.installManagedJsRuntime(true)

    expect(viewmodel.jsRuntimeInstalling).toBe(false)
    expect(basicToast.error).toHaveBeenCalledWith({
      description: 'install failed',
      id: 'js-runtime-install',
      title: 'Failed to install runtime',
    })
  })
})
