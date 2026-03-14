import { EyeIcon, Loader2Icon } from 'lucide-react'
import { useEffect, useState } from 'react'
import { Button } from '@/components/atoms/button'
import { DockviewContentComponent } from '@/components/dockview/content-components'
import { useDockviewLayoutViewModel } from '@/components/dockview/dockview-layout.viewmodel'
import { SessionHeader } from '@/components/panels/session/molecules/session-header'
import { SessionInput } from '@/components/panels/session/molecules/session-input'
import { SessionOutput } from '@/components/panels/session/molecules/session-output'
import { SessionPlanAssignments } from '@/components/panels/session/molecules/session-plan-assignments'
import { useSessionPanelViewmodel } from '@/components/panels/session/session-panel.viewmodel'
// eslint-disable-next-line
import { tauriSessionApi } from '@/lib/api/tauri/session.api'
import type { PlanModel } from '@/lib/models/plan.model'
import { cn } from '@/lib/utils/cn'
import { planPanelId } from '@/lib/utils/dockview-utils'
import { Terminals } from './session-conversation/terminals'

export const SessionPanelLayout = () => {
  const viewmodel = useSessionPanelViewmodel()
  if (!viewmodel.session) return null

  return (
    <div className="flex h-full w-full min-h-0 flex-col overflow-hidden bg-gradient-glow" data-tour="session-view">
      <SessionHeader />
      <SessionPlanAssignments />
      <PlanList />
      <SessionOutput />
      <Terminals messages={viewmodel.terminalBuckets} />
      <SessionInput />
    </div>
  )
}

const PlanList = () => {
  const viewmodel = useSessionPanelViewmodel()
  const plan = viewmodel.plan
  const projectId = viewmodel.session?.projectId
  const sessionId = viewmodel.session?.id

  if (!sessionId || !projectId || !plan) return null

  return (
    <div className="flex flex-col gap-2 mx-2 mt-2 ">
      {plan && <PlanItem plan={plan} projectId={projectId} sessionId={sessionId} />}
    </div>
  )
}

interface PlanItemProps {
  plan: PlanModel
  sessionId: string
  projectId: string
}

const PlanItem = ({ plan, sessionId, projectId }: PlanItemProps) => {
  const viewmodel = useSessionPanelViewmodel()
  const dockviewLayout = useDockviewLayoutViewModel()

  const [isOpen, setIsOpen] = useState(false)
  const isRunning = viewmodel.isRunning && viewmodel.isPlanInProgress
  const shouldContinue = !viewmodel.isRunning && viewmodel.isPlanInProgress

  useEffect(() => {
    const paneId = planPanelId(projectId, plan.id)
    const panel = dockviewLayout.containerApi?.getPanel(paneId)
    setIsOpen(!!panel)

    const unsubscribeRemove = dockviewLayout.containerApi?.onDidRemovePanel((event) => {
      if (event.id === planPanelId(projectId, plan.id)) {
        setIsOpen(false)
      }
    })

    const unsubscribeAdd = dockviewLayout.containerApi?.onDidAddPanel((event) => {
      if (event.id === planPanelId(projectId, plan.id)) {
        setIsOpen(true)
      }
    })

    return () => {
      unsubscribeRemove?.dispose()
      unsubscribeAdd?.dispose()
    }
  }, [dockviewLayout.containerApi, plan.id, projectId])

  const handleBuiltIt = async (plan: PlanModel) => {
    if (plan.isCompletable) {
      plan.setStatus('completed')
      await tauriSessionApi.completePlan(sessionId, plan.id)
    } else if (shouldContinue) {
      viewmodel.session?.setRunning(true)
      plan.setStatus('in_progress')
      try {
        await tauriSessionApi.continuePlanBuild(sessionId, plan.id)
      } catch {
        await tauriSessionApi.startPlanBuild(sessionId, plan.id)
      }
    } else {
      viewmodel.session?.setRunning(true)
      plan.setStatus('in_progress')
      await tauriSessionApi.startPlanBuild(sessionId, plan.id)
    }
  }

  const handleOpenPlan = () => {
    dockviewLayout.openPanel({
      component: DockviewContentComponent.Plan,
      direction: 'right',
      id: planPanelId(projectId, plan.id),
      params: { planId: plan.id, projectId, sessionId },
      title: plan.name,
    })
  }

  const buttonText = plan.isCompletable ? 'Complete' : shouldContinue ? 'Continue' : 'Build It'

  return (
    <div
      key={plan.id}
      className={cn(
        'flex items-center justify-between p-4 border border-amber-400 border-dashed rounded-md bg-accent',
        (plan.isCompletable || isRunning) && 'border-green-700',
      )}
    >
      <div className="flex flex-col gap-1">
        <div className="text-sm font-medium line-clamp-1">{plan.name}:</div>
        <div className="text-sm text-muted-foreground line-clamp-2">{plan.description}</div>
      </div>
      <div className="flex gap-1">
        {!isOpen && (
          <Button size="sm" variant="outline-ghost" onClick={() => handleOpenPlan()}>
            <EyeIcon />
          </Button>
        )}
        <Button
          disabled={!projectId || !sessionId || isRunning}
          size="sm"
          variant="outline-ghost"
          className={cn(
            'border-amber-400 bg-background hover:bg-amber-400 hover:text-background hover:border-black',
            (plan.isCompletable || isRunning) && 'opacity-100! border-green-700',
          )}
          onClick={() => handleBuiltIt(plan)}
        >
          {isRunning && <Loader2Icon className="w-4 h-4 animate-spin" />}
          {!isRunning && buttonText}
        </Button>
      </div>
    </div>
  )
}
