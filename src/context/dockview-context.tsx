import { DockviewReact } from 'dockview-react'
import { EllipsisVertical } from 'lucide-react'
import { type PropsWithChildren, useEffect, useMemo, useState } from 'react'
import { Group, Panel, Separator as ResizableSeparator, useDefaultLayout } from 'react-resizable-panels'
import { ReportBugDialog } from '@/components/dialogs/report-bug-dialog'
import { ReportBugDialogViewModel } from '@/components/dialogs/report-bug-dialog.viewmodel'
import { contentComponents, PanelContainer } from '@/components/dockview/content-components'
import {
  DockviewLayoutViewModel,
  DockviewLayoutViewModelContext,
} from '@/components/dockview/dockview-layout.viewmodel'
import { tabComponents } from '@/components/dockview/tab-components'
import { blprntTheme } from '@/components/dockview/theme'
import { useDockview } from '@/components/dockview/use-dockview'
import { BlprntBanner } from '@/components/organisms/blprnt-banner'
import { BlprntMenu } from '@/components/organisms/blprnt-menu'
import { TourOverlay } from '@/components/organisms/tour-overlay'
import { IntroPanel } from '@/components/panels/intro-panel'
import { useAppViewModel } from '@/hooks/use-app-viewmodel'
import { useBlprntConfig } from '@/lib/utils/blprnt-config'
import { cn } from '@/lib/utils/cn'

export const DockviewProvider = () => {
  const appStore = useAppViewModel()
  const viewmodel = useMemo(() => new DockviewLayoutViewModel(), [])
  const reportBugViewmodel = useMemo(() => new ReportBugDialogViewModel(), [])

  if (!viewmodel) throw new Error('DockviewLayoutViewModel not found')

  if (appStore.isLoading) return null

  return (
    <DockviewLayoutViewModelContext.Provider value={viewmodel}>
      <DockviewProviderInner reportBugViewmodel={reportBugViewmodel} />
    </DockviewLayoutViewModelContext.Provider>
  )
}

export const DockviewProviderInner = ({ reportBugViewmodel }: { reportBugViewmodel: ReportBugDialogViewModel }) => {
  const appViewmodel = useAppViewModel()
  const [showTour, setShowTour] = useState(false)
  const config = useBlprntConfig()

  const { onReady } = useDockview()

  const { defaultLayout, onLayoutChange } = useDefaultLayout({
    id: 'unique-layout-id',
    storage: localStorage,
  })

  useEffect(() => {
    if (config.seenTour) return
    setTimeout(() => setShowTour(true), 500)
  }, [config.seenTour])

  return (
    <>
      {showTour && <TourOverlay onComplete={() => config.setSeenTour(true)} />}
      <div className={cn('flex flex-col w-screen h-screen justify-between')} data-tour="complete-tour">
        <ReportBugDialog viewmodel={reportBugViewmodel} />
        <BlprntBanner />

        <Group
          className={cn('w-screen h-full')}
          defaultLayout={defaultLayout}
          orientation="horizontal"
          onLayoutChange={onLayoutChange}
        >
          <Panel
            className="p-2 pr-0"
            data-tour="sidebar"
            defaultSize="300px"
            id="menu"
            maxSize={appViewmodel.isSidebarExpanded ? '700px' : '54px'}
            minSize={appViewmodel.isSidebarExpanded ? '300px' : '54px'}
          >
            <BlprntMenu reportBugViewmodel={reportBugViewmodel} />
          </Panel>

          {appViewmodel.isSidebarExpanded && <SeparatorSidebar />}

          <Panel className="p-2 pl-0 h-full" id="content">
            <div className={cn('h-full flex bg-background rounded-lg border overflow-hidden')}>
              <DockviewReact
                components={contentComponents}
                tabComponents={tabComponents}
                theme={blprntTheme}
                watermarkComponent={() => (
                  <PanelContainer>
                    <IntroPanel />
                  </PanelContainer>
                )}
                onReady={onReady}
              />
            </div>
          </Panel>
        </Group>
      </div>
    </>
  )
}

const SeparatorSidebar = ({
  className = '',
}: PropsWithChildren<{
  className?: string
}>) => {
  return (
    <ResizableSeparator
      className={cn(
        'w-2 flex items-center justify-center transition-colors duration-300 relative my-2',
        'bg-transparent cursor-col-resize!',
        "text-transparent data-[separator='hover']:text-primary-foreground data-[separator='active']:text-primary-foreground",
        "[&[data-separator='hover']>div]:h-6 [&[data-separator='hover']>div]:w-4",
        "[&[data-separator='active']>div]:h-6 [&[data-separator='active']>div]:w-4",
        className,
      )}
    >
      <div className="flex items-center justify-center h-24 w-1 absolute z-50 border transition-all duration-300 rounded-xs bg-linear-to-b from-chart-5 via-primary to-chart-5 pointer-events-none">
        <EllipsisVertical />
      </div>
    </ResizableSeparator>
  )
}
