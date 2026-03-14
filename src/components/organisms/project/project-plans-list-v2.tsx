import { AlertCircle, ListTodo, Pencil, Trash2 } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { useEffect, useMemo, useState } from 'react'
import type { PlanDocumentStatus, ProjectPlanListItem } from '@/bindings'
import { Button } from '@/components/atoms/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/atoms/card'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/atoms/dialog'
import { Empty, EmptyContent, EmptyHeader, EmptyMedia, EmptyTitle } from '@/components/atoms/empty'
import { Input } from '@/components/atoms/input'
import { Label } from '@/components/atoms/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/atoms/select'
import { Skeleton } from '@/components/atoms/skeleton'
import { basicToast } from '@/components/atoms/toaster'
import { TooltipMacro } from '@/components/atoms/tooltip-macro'
import { DockviewContentComponent } from '@/components/dockview/content-components'
import { useDockviewLayoutViewModel } from '@/components/dockview/dockview-layout.viewmodel'
import { Section } from '@/components/organisms/page/section'
import { EventType, globalEventBus } from '@/lib/events'
import { PlanModel } from '@/lib/models/plan.model'
import { cn } from '@/lib/utils/cn'
import { planPanelId } from '@/lib/utils/dockview-utils'
import { type PlanSortOption, type PlanStatusFilter, ProjectPlansListViewModel } from './plan-list-viewmodel'

interface ProjectPlansListProps {
  projectId: string
}

export const ProjectPlansListV2 = observer(({ projectId }: ProjectPlansListProps) => {
  const dockviewLayout = useDockviewLayoutViewModel()
  const viewmodel = useMemo(() => new ProjectPlansListViewModel(projectId), [projectId])
  const [actionPlan, setActionPlan] = useState<ProjectPlanListItem | null>(null)
  const [actionType, setActionType] = useState<'archive' | 'delete' | null>(null)
  const [isActionSubmitting, setIsActionSubmitting] = useState(false)

  useEffect(() => {
    void viewmodel.init()
    const unsubscribe = globalEventBus.subscribe(EventType.Internal, (event) => {
      if (event.payload.event.type !== 'plan_updated') return
      if (event.payload.event.projectId !== projectId) return
      void viewmodel.load()
    })

    return () => {
      unsubscribe()
      viewmodel.destroy()
    }
  }, [projectId, viewmodel])

  const openPlanPanel = (planId: string, title: string) => {
    dockviewLayout.openPanel({
      component: DockviewContentComponent.Plan,
      direction: 'right',
      id: planPanelId(projectId, planId),
      params: { planId, projectId },
      title,
    })
  }

  const openEditFlow = (plan: ProjectPlanListItem) => {
    openPlanPanel(plan.id, plan.name)
  }

  const openDeleteDialog = (plan: ProjectPlanListItem) => {
    setActionPlan(plan)
    setActionType('delete')
  }

  const closeActionDialog = () => {
    if (isActionSubmitting) return
    setActionPlan(null)
    setActionType(null)
  }

  const confirmAction = async () => {
    if (!actionPlan || !actionType) return

    setIsActionSubmitting(true)
    try {
      if (actionType === 'archive') {
        await PlanModel.archiveForProject(projectId, actionPlan.id)
        basicToast.success({ title: 'Plan archived' })
      } else {
        await PlanModel.deleteForProject(projectId, actionPlan.id)
        basicToast.success({ title: 'Plan deleted' })
      }

      await viewmodel.load()
      setActionPlan(null)
      setActionType(null)
    } catch (error) {
      const message = error instanceof Error ? error.message : `Failed to ${actionType} plan`
      basicToast.error({
        description: message,
        title: actionType === 'archive' ? 'Failed to archive plan' : 'Failed to delete plan',
      })
    } finally {
      setIsActionSubmitting(false)
    }
  }

  const controls = (
    <div className="flex flex-wrap items-end gap-3 py-4">
      <div className="min-w-[220px] flex-1 max-w-sm">
        <Label className="text-xs text-muted-foreground mb-1.5">Search</Label>
        <Input
          placeholder="Search plans"
          value={viewmodel.search}
          onChange={(event) => viewmodel.setSearch(event.target.value)}
        />
      </div>

      <div className="flex flex-col gap-1.5">
        <Label className="text-xs text-muted-foreground">Sort</Label>
        <Select value={viewmodel.sort} onValueChange={(value) => viewmodel.setSort(value as PlanSortOption)}>
          <SelectTrigger className="w-52">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="updated_desc">Recently updated</SelectItem>
            <SelectItem value="updated_asc">Least recently updated</SelectItem>
            <SelectItem value="created_desc">Newest created</SelectItem>
            <SelectItem value="created_asc">Oldest created</SelectItem>
            <SelectItem value="name_asc">Name (A-Z)</SelectItem>
            <SelectItem value="name_desc">Name (Z-A)</SelectItem>
          </SelectContent>
        </Select>
      </div>

      <div className="flex flex-col gap-1.5">
        <Label className="text-xs text-muted-foreground">Status</Label>
        <Select
          value={viewmodel.statusFilter}
          onValueChange={(value) => viewmodel.setStatusFilter(value as PlanStatusFilter)}
        >
          <SelectTrigger className="w-40">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All statuses</SelectItem>
            {viewmodel.statusOptions.map((status) => (
              <SelectItem key={status} value={status}>
                {viewmodel.getStatusLabel(status)}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
    </div>
  )

  const bulkToolbar = viewmodel.selectedCount > 0 && (
    <div className="flex flex-wrap items-center gap-3 rounded-md border border-border bg-card px-3 py-2">
      <div className="text-sm text-muted-foreground">{viewmodel.selectedCount} selected</div>
      <Select
        value={viewmodel.bulkStatus ?? ''}
        onValueChange={(value) => viewmodel.setBulkStatus(value as PlanDocumentStatus)}
      >
        <SelectTrigger className="w-44" size="sm">
          <SelectValue placeholder="Set status" />
        </SelectTrigger>
        <SelectContent>
          {viewmodel.statusOptions.map((status) => (
            <SelectItem key={status} value={status}>
              {viewmodel.getStatusLabel(status)}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      <Button
        disabled={!viewmodel.bulkStatus || viewmodel.isBulkUpdating}
        size="sm"
        variant="outline"
        onClick={() => void viewmodel.applyBulkStatus()}
      >
        Apply
      </Button>
    </div>
  )

  if (viewmodel.isLoading) {
    return (
      <Section>
        {controls}
        <div className="grid gap-3 py-4">
          <Skeleton className="h-28 w-full" />
          <Skeleton className="h-28 w-full" />
          <Skeleton className="h-28 w-full" />
        </div>
      </Section>
    )
  }

  if (viewmodel.error) {
    return (
      <Section>
        {controls}
        <Empty className="min-h-[280px] py-8">
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <AlertCircle className="size-5" />
            </EmptyMedia>
            <EmptyTitle>Failed to load plans</EmptyTitle>
          </EmptyHeader>
          <EmptyContent>
            <div className="text-sm text-muted-foreground">{viewmodel.error}</div>
            <Button size="sm" variant="outline" onClick={() => void viewmodel.load()}>
              Retry
            </Button>
          </EmptyContent>
        </Empty>
      </Section>
    )
  }

  if (viewmodel.plans.length === 0) {
    return (
      <Section>
        {controls}
        <Empty className="min-h-[280px] py-8">
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <ListTodo className="size-5" />
            </EmptyMedia>
            <EmptyTitle>No plans found</EmptyTitle>
          </EmptyHeader>
          <EmptyContent>
            <div className="text-sm text-muted-foreground">No plans match the current filters.</div>
          </EmptyContent>
        </Empty>
      </Section>
    )
  }

  return (
    <Section>
      {controls}
      {bulkToolbar}
      <div className="grid gap-3 py-4">
        {viewmodel.plans.map((plan) => (
          <Card
            key={plan.id}
            className={cn(
              'gap-3 py-4 cursor-pointer hover:bg-muted',
              viewmodel.selectedPlanIds.has(plan.id) && 'border-dashed border-foreground',
            )}
            onClick={() => viewmodel.togglePlanSelected(plan.id)}
          >
            <CardHeader className="px-4 flex items-start justify-between gap-3">
              <CardTitle className="text-sm">{plan.name}</CardTitle>
              <div className="flex flex-wrap gap-2 shrink-0">
                <Select
                  value={plan.status ?? 'pending'}
                  onValueChange={(value) => void viewmodel.updatePlanStatus(plan.id, value as PlanDocumentStatus)}
                >
                  <SelectTrigger className="w-40" size="sm">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {viewmodel.statusOptions.map((status) => (
                      <SelectItem key={status} value={status}>
                        {viewmodel.getStatusLabel(status)}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <TooltipMacro tooltip="Edit plan">
                  <Button size="sm" variant="outline" onClick={() => openEditFlow(plan)}>
                    <Pencil />
                  </Button>
                </TooltipMacro>
                <TooltipMacro tooltip="Delete plan">
                  <Button size="sm" variant="destructive" onClick={() => openDeleteDialog(plan)}>
                    <Trash2 />
                  </Button>
                </TooltipMacro>
              </div>
            </CardHeader>
            <CardContent className="px-4 text-sm text-muted-foreground">
              {plan.description || 'No description'}
            </CardContent>
          </Card>
        ))}
      </div>

      <Dialog open={!!actionPlan && !!actionType} onOpenChange={(isOpen) => !isOpen && closeActionDialog()}>
        <DialogContent className="max-w-lg">
          <DialogHeader>
            <DialogTitle>{actionType === 'archive' ? 'Archive Plan' : 'Delete Plan'}</DialogTitle>
            <DialogDescription>
              {actionType === 'archive'
                ? 'Are you sure you want to archive this plan? This will detach it from its current session.'
                : 'Are you sure you want to delete this plan? This action cannot be undone and will detach it from its current session.'}
            </DialogDescription>
          </DialogHeader>

          <div className="rounded-md border border-border p-3 text-sm">
            <div className="font-medium text-foreground">{actionPlan?.name}</div>
            <div className="text-muted-foreground">{actionPlan?.description || 'No description'}</div>
          </div>

          <DialogFooter>
            <Button disabled={isActionSubmitting} size="sm" variant="outline" onClick={closeActionDialog}>
              Back
            </Button>
            <Button
              disabled={isActionSubmitting}
              size="sm"
              variant={actionType === 'delete' ? 'destructive' : 'outline'}
              onClick={() => void confirmAction()}
            >
              {actionType === 'archive' ? 'Confirm archive' : 'Confirm delete'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </Section>
  )
})
