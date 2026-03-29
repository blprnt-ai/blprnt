import { Link, useNavigate, useRouterState } from '@tanstack/react-router'
import { BotIcon, HomeIcon, KanbanIcon, PenLine, PlusIcon, TimerIcon, Trash2Icon, UserIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { useState } from 'react'
import { toast } from 'sonner'
import { useAppViewmodel } from '@/app.viewmodel'
import { IssueForm } from '@/components/forms/issue'
import { IssueFormViewmodel } from '@/components/forms/issue/issue-form.viewmodel'
import { Button } from '@/components/ui/button'
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupAction,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
  useSidebar,
} from '@/components/ui/sidebar'
import { projectsApi } from '@/lib/api/projects'
import { formatRunTime } from '@/lib/runs'
import { AppModel } from '@/models/app.model'
import type { ColorVariant } from '../ui/colors'
import { TextColoredSpan } from '../ui/colors'
import { employeeIconValueToIcon } from '../ui/employee-label'
import { RunStatusChip } from './run-status-chip'

export const AppSidebar = observer(() => {
  const appViewmodel = useAppViewmodel()
  const pathname = useRouterState({ select: (state) => state.location.pathname })
  const navigate = useNavigate()
  const { open } = useSidebar()
  const [isNukingDatabase, setIsNukingDatabase] = useState(false)
  const [issueFormViewmodel] = useState(
    () =>
      new IssueFormViewmodel(async (issue) => {
        await navigate({
          params: { issueId: issue.id },
          to: '/issues/$issueId',
        })
      }),
  )
  const isDev = import.meta.env.DEV

  const isActive = (path: string) => pathname === path
  const handleNukeDatabase = async () => {
    if (!window.confirm('Nuke the local database and restart onboarding? This cannot be undone.')) return

    setIsNukingDatabase(true)

    try {
      await projectsApi.nukeDatabase()
      AppModel.instance.resetAfterDatabaseNuke()
      toast.success('Database nuked. Redirecting to onboarding.')
      window.location.assign('/onboarding')
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to nuke database.')
    } finally {
      setIsNukingDatabase(false)
    }
  }

  return (
    <Sidebar collapsible="icon" variant="floating">
      <SidebarHeader className="list-none group-data-[collapsible=icon]:hidden">
        <SidebarMenuItem>
          <SidebarMenuButton className="text-primary" variant="outline" onClick={issueFormViewmodel.open}>
            <PenLine /> New Issue
          </SidebarMenuButton>
        </SidebarMenuItem>
        <SidebarMenuItem>
          <Link to="/">
            <SidebarMenuButton isActive={isActive('/')}>
              <HomeIcon /> Dashboard
            </SidebarMenuButton>
          </Link>
        </SidebarMenuItem>
        <SidebarMenuItem>
          <Link to="/issues">
            <SidebarMenuButton isActive={isActive('/issues')}>
              <KanbanIcon /> Issues
            </SidebarMenuButton>
          </Link>
        </SidebarMenuItem>
        <SidebarMenuItem>
          <Link to="/projects">
            <SidebarMenuButton isActive={isActive('/projects')}>
              <BotIcon /> Projects
            </SidebarMenuButton>
          </Link>
        </SidebarMenuItem>
        <SidebarMenuItem>
          <Link to="/employees">
            <SidebarMenuButton isActive={isActive('/employees')}>
              <UserIcon /> Employees
            </SidebarMenuButton>
          </Link>
        </SidebarMenuItem>
        <SidebarMenuItem>
          <Link to="/runs">
            <SidebarMenuButton isActive={isActive('/runs')}>
              <TimerIcon /> Runs
            </SidebarMenuButton>
          </Link>
        </SidebarMenuItem>
      </SidebarHeader>

      <SidebarContent>
        <SidebarGroup className="hidden group-data-[collapsible=icon]:flex">
          <SidebarMenuButton className="text-primary" variant="outline" onClick={issueFormViewmodel.open}>
            <PenLine /> New Issue
          </SidebarMenuButton>
        </SidebarGroup>

        <SidebarGroup className="hidden group-data-[collapsible=icon]:flex">
          <Link to="/">
            <SidebarMenuButton isActive={isActive('/')}>
              <HomeIcon /> Dashboard
            </SidebarMenuButton>
          </Link>
        </SidebarGroup>

        <SidebarGroup className="hidden group-data-[collapsible=icon]:flex">
          <Link to="/issues">
            <SidebarMenuButton isActive={isActive('/issues')}>
              <KanbanIcon /> Issues
            </SidebarMenuButton>
          </Link>
        </SidebarGroup>

        <SidebarGroup className="hidden group-data-[collapsible=icon]:flex">
          <Link to="/projects">
            <SidebarMenuButton isActive={isActive('/projects')}>
              <BotIcon /> Projects
            </SidebarMenuButton>
          </Link>
        </SidebarGroup>

        <SidebarGroup className="hidden group-data-[collapsible=icon]:flex">
          <Link to="/employees">
            <SidebarMenuButton isActive={isActive('/employees')}>
              <UserIcon /> Employees
            </SidebarMenuButton>
          </Link>
        </SidebarGroup>

        <SidebarGroup className="hidden group-data-[collapsible=icon]:flex">
          <Link to="/runs">
            <SidebarMenuButton isActive={isActive('/runs')}>
              <TimerIcon /> Runs
            </SidebarMenuButton>
          </Link>
        </SidebarGroup>

        <SidebarGroup>
          <SidebarGroupLabel>
            <Link to="/runs">Runs</Link>
          </SidebarGroupLabel>

          <SidebarGroupContent>
            {!open && (
              <SidebarMenuItem>
                <SidebarMenuButton isActive={isActive('/runs')}>
                  <TimerIcon />
                  Runs
                </SidebarMenuButton>
              </SidebarMenuItem>
            )}
            {open &&
              appViewmodel.runs.runningRuns.map((run) => (
                <SidebarMenuItem key={run.id}>
                  <Link params={{ runId: run.id }} to="/runs/$runId">
                    <SidebarMenuButton isActive={isActive(`/runs/${run.id}`)}>
                      <div className="min-w-0 flex-1 space-y-1">
                        <div className="truncate text-sm font-medium">
                          {AppModel.instance.resolveEmployeeName(run.employeeId) ?? 'Unknown employee'}
                        </div>
                        <div className="flex items-center gap-2 text-xs text-muted-foreground">
                          <RunStatusChip status={run.status} />
                          <span>{formatRunTime(run.startedAt ?? run.createdAt)}</span>
                        </div>
                      </div>
                    </SidebarMenuButton>
                  </Link>
                </SidebarMenuItem>
              ))}
            {open && appViewmodel.runs.runningRuns.length === 0 && (
              <SidebarMenuItem>
                <SidebarMenuButton>
                  <TimerIcon />
                  No active runs
                </SidebarMenuButton>
              </SidebarMenuItem>
            )}
          </SidebarGroupContent>
        </SidebarGroup>

        <SidebarGroup>
          <SidebarGroupLabel>Projects</SidebarGroupLabel>
          <SidebarGroupAction>
            <PlusIcon />
          </SidebarGroupAction>

          <SidebarGroupContent>
            {!open && (
              <SidebarMenuItem>
                <Link to="/projects">
                  <SidebarMenuButton isActive={isActive('/projects')}>
                    <BotIcon />
                    Projects
                  </SidebarMenuButton>
                </Link>
              </SidebarMenuItem>
            )}
            {open &&
              AppModel.instance.projects.map((project) => (
                <SidebarMenuItem key={project.id}>
                  <Link params={{ projectId: project.id }} to="/projects/$projectId">
                    <SidebarMenuButton isActive={isActive(`/projects/${project.id}`)}>{project.name}</SidebarMenuButton>
                  </Link>
                </SidebarMenuItem>
              ))}
          </SidebarGroupContent>
        </SidebarGroup>

        <SidebarGroup>
          <SidebarGroupLabel>Employees</SidebarGroupLabel>
          <SidebarGroupAction>
            <PlusIcon />
          </SidebarGroupAction>

          <SidebarGroupContent>
            {open &&
              AppModel.instance.employees.map((employee) => {
                const Icon = employeeIconValueToIcon(employee.icon!)

                return (
                  <SidebarMenuItem key={employee.id}>
                    <Link params={{ employeeId: employee.id }} to="/employees/$employeeId">
                      <SidebarMenuButton isActive={isActive(`/employees/${employee.id}`)}>
                        <TextColoredSpan color={employee.color as ColorVariant}>
                          <Icon />
                        </TextColoredSpan>
                        {employee.name}
                      </SidebarMenuButton>
                    </Link>
                  </SidebarMenuItem>
                )
              })}
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter>
        {isDev && (
          <Button
            aria-label="Nuke database"
            className="w-full justify-start group-data-[collapsible=icon]:size-8 group-data-[collapsible=icon]:justify-center"
            disabled={isNukingDatabase}
            type="button"
            variant="destructive-outline"
            onClick={() => void handleNukeDatabase()}
          >
            <Trash2Icon />
            {open && (isNukingDatabase ? 'Nuking database...' : 'Nuke Database')}
          </Button>
        )}
      </SidebarFooter>
      <IssueForm viewmodel={issueFormViewmodel} />
      <SidebarRail />
    </Sidebar>
  )
})
