import type { IDockviewPanelProps } from 'dockview-react'
import type { PropsWithChildren } from 'react'
import { ErrorBoundary } from '@/components/organisms/error-boundary'
import { IntroPanel } from '@/components/panels/intro-panel'
import { NewProjectPanel } from '@/components/panels/new-project-panel'
import { PersonalityPanel } from '@/components/panels/personality-panel'
import { PlanPanel } from '@/components/panels/plan'
import { PreviewPanel } from '@/components/panels/preview-panel'
import { ProjectPanel } from '@/components/panels/project-panel'
import { SessionPanel } from '@/components/panels/session'
import { SettingsPage, type SettingsTabs } from '@/components/views/settings/settings-page'
import { cn } from '@/lib/utils/cn'
import { newProjectId } from '@/lib/utils/default-models'

export enum DockviewContentComponent {
  Intro = 'intro',
  Personality = 'personality',
  Session = 'session',
  Project = 'project',
  Plan = 'plan',

  UserAccount = 'user-account',
  Preview = 'preview',
}

interface DockviewPanelProps extends Omit<IDockviewPanelProps, 'params'> {
  params: {
    sessionId?: string
    projectId?: string
    planId?: string
    tab?: SettingsTabs
  }
}

export const contentComponents = {
  [DockviewContentComponent.Session]: (props: DockviewPanelProps) => {
    if (!props.params.sessionId) return null

    const handleClose = () => props.api.close()

    return (
      <PanelContainer withPadding={false}>
        <ErrorBoundary action={handleClose} title="Error rendering session">
          <SessionPanel sessionId={props.params.sessionId} />
        </ErrorBoundary>
      </PanelContainer>
    )
  },
  [DockviewContentComponent.Project]: (props: DockviewPanelProps) => {
    if (props.params.projectId === undefined) return null

    const handleClose = () => props.api.close()

    return (
      <PanelContainer withPadding={false}>
        <ErrorBoundary action={handleClose} title="Error rendering project">
          {props.params.projectId === newProjectId ? (
            <NewProjectPanel />
          ) : (
            <ProjectPanel projectId={props.params.projectId} />
          )}
        </ErrorBoundary>
      </PanelContainer>
    )
  },
  [DockviewContentComponent.Plan]: (props: DockviewPanelProps) => {
    if (!props.params.projectId || !props.params.planId) return null

    return (
      <PanelContainer withPadding={false}>
        <PlanPanel planId={props.params.planId} projectId={props.params.projectId} />
      </PanelContainer>
    )
  },
  [DockviewContentComponent.Intro]: (props: DockviewPanelProps) => {
    const handleClose = () => props.api.close()

    return (
      <PanelContainer>
        <ErrorBoundary action={handleClose} title="Error rendering intro">
          <IntroPanel />
        </ErrorBoundary>
      </PanelContainer>
    )
  },
  [DockviewContentComponent.Personality]: (props: DockviewPanelProps) => {
    const handleClose = () => props.api.close()

    return (
      <PanelContainer>
        <ErrorBoundary action={handleClose} title="Error rendering personality">
          <PersonalityPanel />
        </ErrorBoundary>
      </PanelContainer>
    )
  },
  [DockviewContentComponent.UserAccount]: (props: DockviewPanelProps) => {
    const tab = props.params.tab ?? 'models'

    return (
      <PanelContainer withPadding={false}>
        <SettingsPage initialTab={tab} />
      </PanelContainer>
    )
  },
  [DockviewContentComponent.Preview]: (props: DockviewPanelProps) => {
    if (!props.params.projectId) return null

    return (
      <PanelContainer withPadding={false}>
        <PreviewPanel projectId={props.params.projectId} />
      </PanelContainer>
    )
  },
}

interface PanelContainerProps extends PropsWithChildren {
  withPadding?: boolean
}

export const PanelContainer = ({ children, withPadding = true }: PanelContainerProps) => {
  return <div className={cn('h-full w-full bg-gradient-glow', withPadding && 'p-4')}>{children}</div>
}
