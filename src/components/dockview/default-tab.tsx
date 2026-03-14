import { DockviewDefaultTab, type IDockviewPanelHeaderProps } from 'dockview-react'
import { useDockviewLayoutViewModel } from '@/components/dockview/dockview-layout.viewmodel'

export const DefaultTab = (props: IDockviewPanelHeaderProps) => {
  const dockviewLayout = useDockviewLayoutViewModel()

  return (
    <DockviewDefaultTab
      {...props}
      closeActionOverride={() => {
        void dockviewLayout.closePanel(props.api.id)
      }}
    />
  )
}
