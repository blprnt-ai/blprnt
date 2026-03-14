import type { DockviewApi, DockviewReadyEvent } from 'dockview-react'
import { useCallback, useEffect, useState } from 'react'
import { useHotkeys } from 'react-hotkeys-hook'
import { DockviewContentComponent } from '@/components/dockview/content-components'
import { useDockviewLayoutViewModel } from '@/components/dockview/dockview-layout.viewmodel'

interface UseDockviewReturn {
  onReady: (event: DockviewReadyEvent) => void
}

export const useDockview = (): UseDockviewReturn => {
  const dockviewLayout = useDockviewLayoutViewModel()

  const [containerApi, setContainerApi] = useState<DockviewApi | null>(null)

  const onReady = useCallback(
    (event: DockviewReadyEvent) => {
      setContainerApi(event.api)
      dockviewLayout.onReady(event)
    },
    [dockviewLayout],
  )

  useEffect(() => {
    if (!containerApi) return

    const disposable = containerApi.onDidLayoutChange(() => {
      dockviewLayout.saveLayoutToStorage()
    })

    return () => disposable.dispose()
  }, [containerApi, dockviewLayout])

  useEffect(() => {
    if (!containerApi) return

    const disposable = containerApi.onDidActivePanelChange((event) =>
      dockviewLayout.setActivePanelId(event?.id ?? null),
    )

    return () => disposable.dispose()
  }, [containerApi, dockviewLayout])

  useHotkeys(
    ['ctrl+t', 'meta+t'],
    () => {
      if (!containerApi) return

      containerApi.addPanel({
        component: DockviewContentComponent.Intro,
        id: 'intro',
        title: 'Intro',
      })
    },
    {},
    [containerApi],
  )

  useHotkeys(
    ['ctrl+w', 'meta+w'],
    () => {
      if (!containerApi) return

      const panelId = containerApi.activePanel?.id || null
      if (!panelId) return

      void dockviewLayout.closePanel(panelId)
    },
    {},
    [containerApi, dockviewLayout],
  )

  return { onReady }
}
