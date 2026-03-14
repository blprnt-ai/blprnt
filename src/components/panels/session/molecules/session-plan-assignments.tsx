import { ArrowLeftRight, CheckIcon, PlusIcon } from 'lucide-react'
import { useEffect, useState } from 'react'
import type { ProjectPlanListItem, TauriError } from '@/bindings'
import { Button } from '@/components/atoms/button'
import { Checkbox } from '@/components/atoms/checkbox'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/atoms/dialog'
import { ScrollArea } from '@/components/atoms/scroll-area'
import { Skeleton } from '@/components/atoms/skeleton'
import { basicToast } from '@/components/atoms/toaster'
import { useSessionPanelViewmodel } from '@/components/panels/session/session-panel.viewmodel'
import { PlanModel } from '@/lib/models/plan.model'
import { errorToMessage } from '@/lib/utils/misc'

interface AssignmentOption {
  id: string
  name: string
}

export const SessionPlanAssignments = () => {
  const viewmodel = useSessionPanelViewmodel()
  const session = viewmodel.session
  const projectId = session?.projectId
  const sessionId = session?.id

  const [isOpen, setIsOpen] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [isSaving, setIsSaving] = useState(false)
  const [options, setOptions] = useState<AssignmentOption[]>([])
  const [selectedPlanId, setSelectedPlanId] = useState<string | null>(null)

  const loadPlans = async () => {
    if (!projectId) return

    try {
      const [plans] = await Promise.all([
        PlanModel.listForProject(projectId, { status_filter: ['pending', 'in_progress', 'completed'] }),
      ])

      setOptions(
        plans
          .filter((plan) => plan.status !== 'archived' && plan.status !== 'completed')
          .map((plan: ProjectPlanListItem) => ({
            id: plan.id,
            name: plan.name,
          })),
      )

      const attachedPlan = plans.find((plan: ProjectPlanListItem) => plan.parent_session_id === sessionId)
      if (attachedPlan) {
        setSelectedPlanId(attachedPlan.id)
        const plan = await PlanModel.get(projectId, attachedPlan.id)
        viewmodel.setPlan(plan)
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to load plans'
      basicToast.error({ description: message, title: 'Failed to load plans' })
    }
  }

  // biome-ignore lint/correctness/useExhaustiveDependencies: we only want to load plans once
  useEffect(() => {
    if (!projectId || !sessionId) return

    void loadPlans()
  }, [])

  if (!projectId || !sessionId || !options.length) return null

  const openDialog = async () => {
    setIsOpen(true)
    setIsLoading(true)
    setSelectedPlanId(viewmodel.session?.plan?.id ?? null)

    try {
      const [plans] = await Promise.all([
        PlanModel.listForProject(projectId, { status_filter: ['pending', 'in_progress', 'completed'] }),
      ])

      setOptions(
        plans.map((plan: ProjectPlanListItem) => ({
          id: plan.id,
          name: plan.name,
        })),
      )
      const attachedPlan = plans.find((plan: ProjectPlanListItem) => plan.parent_session_id === sessionId)
      if (attachedPlan) {
        setSelectedPlanId(attachedPlan.id)
        const plan = await PlanModel.get(projectId, attachedPlan.id)
        viewmodel.setPlan(plan)
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to load plans'
      basicToast.error({ description: message, title: 'Failed to load plans' })
      setIsOpen(false)
    } finally {
      setIsLoading(false)
    }
  }

  const closeDialog = () => {
    if (isSaving) return
    setIsOpen(false)
    setSelectedPlanId(null)
  }

  const selectOption = (planId: string, isChecked: boolean) => {
    setSelectedPlanId(isChecked ? planId : null)
  }

  const applyAssignments = async () => {
    if (!viewmodel.session) return
    setIsSaving(true)

    try {
      await viewmodel.setSessionPlanAssignment(selectedPlanId)
      const plan = selectedPlanId ? await PlanModel.get(projectId, selectedPlanId) : null
      basicToast.success({ title: 'Session plan updated' })
      viewmodel.setPlan(plan)

      setIsOpen(false)
      setSelectedPlanId(null)
    } catch (error) {
      const err = error as TauriError

      if (err satisfies TauriError) {
        basicToast.error(errorToMessage(err))
      } else {
        const message = error instanceof Error ? error.message : 'Failed to update session plan'
        basicToast.error({ description: message, title: 'Failed to update session plan' })
      }
    } finally {
      setIsSaving(false)
    }
  }

  return (
    <>
      <div className="mx-2 mt-2">
        <Button className="w-full justify-between" size="sm" variant="outline" onClick={() => void openDialog()}>
          <span className="inline-flex items-center gap-2 text-muted-foreground font-light">
            {viewmodel.hasPlan ? (
              <ArrowLeftRight className="size-4 use-stroke-width" strokeWidth={1} />
            ) : (
              <PlusIcon className="size-4 use-stroke-width" strokeWidth={1} />
            )}
            {viewmodel.hasPlan ? 'Switch Plan' : 'Attach Plan'}
          </span>
        </Button>
      </div>

      <Dialog open={isOpen} onOpenChange={(open) => !open && closeDialog()}>
        <DialogContent className="max-w-4xl">
          <DialogHeader>
            <DialogTitle>Attach plan to session</DialogTitle>
            <DialogDescription>Select a plan to attach, or clear selection to detach.</DialogDescription>
          </DialogHeader>

          {isLoading ? (
            <div className="grid gap-2 py-2">
              <Skeleton className="h-10 w-full" />
              <Skeleton className="h-10 w-full" />
              <Skeleton className="h-10 w-full" />
            </div>
          ) : options.length === 0 ? (
            <div className="rounded-md border border-border p-3 text-sm text-muted-foreground">
              No plans available in this project.
            </div>
          ) : (
            <ScrollArea className="max-h-72 rounded-md border border-border">
              <div className="divide-y divide-border">
                {options.map((option) => (
                  <label key={option.id} className="flex cursor-pointer items-center gap-3 px-3 py-2 text-sm">
                    <Checkbox
                      checked={selectedPlanId === option.id}
                      onCheckedChange={(checked) => selectOption(option.id, checked === true)}
                    />
                    <span className="truncate flex-1">{option.name}</span>
                    {selectedPlanId === option.id && <CheckIcon className="size-4 text-muted-foreground" />}
                  </label>
                ))}
              </div>
            </ScrollArea>
          )}

          <DialogFooter>
            <Button disabled={isSaving} size="sm" variant="outline" onClick={closeDialog}>
              Cancel
            </Button>
            <Button
              disabled={isLoading || isSaving}
              size="sm"
              variant="outline"
              onClick={() => void applyAssignments()}
            >
              Apply
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  )
}
