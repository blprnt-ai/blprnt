import { Link, useNavigate, useRouterState } from '@tanstack/react-router'
import {
  BotIcon,
  HomeIcon,
  KanbanIcon,
  LogOutIcon,
  ListTodoIcon,
  PenLine,
  PlusIcon,
  SlidersHorizontalIcon,
  TimerIcon,
  Trash2Icon,
  UserIcon,
} from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { useState } from 'react'
import { toast } from 'sonner'
import { useAppViewmodel } from '@/app.viewmodel'
import { EmployeeForm } from '@/components/forms/employee'
import { EmployeeFormViewmodel } from '@/components/forms/employee/employee-form.viewmodel'
import { IssueForm } from '@/components/forms/issue'
import { IssueFormViewmodel } from '@/components/forms/issue/issue-form.viewmodel'
import { ProjectForm } from '@/components/forms/project'
import { ProjectFormViewmodel } from '@/components/forms/project/project-form.viewmodel'
import { ConfirmationDialog } from '@/components/molecules/confirmation-dialog'
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

export const AppSidebar = observer(() => {
  const appViewmodel = useAppViewmodel()
  const pathname = useRouterState({ select: (state) => state.location.pathname })
  const navigate = useNavigate()
  const { isMobile, open, setOpenMobile } = useSidebar()
  const [isNukingDatabase, setIsNukingDatabase] = useState(false)
  const [isNukeDialogOpen, setIsNukeDialogOpen] = useState(false)
  const [issueFormViewmodel] = useState(
    () =>
      new IssueFormViewmodel(async (issue) => {
        await navigate({
          params: { issueId: issue.id },
          to: '/issues/$issueId',
        })
      }),
  )
  const [projectFormViewmodel] = useState(
    () =>
      new ProjectFormViewmodel(async (project) => {
        await navigate({
          params: { projectId: project.id },
          to: '/projects/$projectId',
        })
      }),
  )
  const [employeeFormViewmodel] = useState(
    () =>
      new EmployeeFormViewmodel(async (employee) => {
        await navigate({
          params: { employeeId: employee.id },
          to: '/employees/$employeeId',
        })
      }),
  )
  const isDev = import.meta.env.DEV

  const isActive = (path: string) => pathname === path
  const closeMobileSidebar = () => {
    if (isMobile) setOpenMobile(false)
  }

  const handleNukeDatabase = async () => {
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

  const handleLogout = async () => {
    await appViewmodel.logout()
    closeMobileSidebar()
    window.location.assign('/login')
  }

  return (
    <>
      <Sidebar collapsible="icon" variant="floating">
        <SidebarHeader className="list-none border-b border-sidebar-border/70 group-data-[collapsible=icon]:hidden">
          <SidebarMenuItem>
            <SidebarMenuButton
              className="text-primary"
              variant="outline"
              onClick={() => {
                issueFormViewmodel.open()
                closeMobileSidebar()
              }}
            >
              <PenLine /> New Issue
            </SidebarMenuButton>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <Link to="/" onClick={closeMobileSidebar}>
              <SidebarMenuButton isActive={isActive('/')}>
                <HomeIcon /> Dashboard
              </SidebarMenuButton>
            </Link>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <Link to="/my-work" onClick={closeMobileSidebar}>
              <SidebarMenuButton isActive={isActive('/my-work')}>
                <ListTodoIcon /> My Work
              </SidebarMenuButton>
            </Link>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <Link to="/issues" onClick={closeMobileSidebar}>
              <SidebarMenuButton isActive={isActive('/issues')}>
                <KanbanIcon /> Issues
              </SidebarMenuButton>
            </Link>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <Link to="/runs" onClick={closeMobileSidebar}>
              <SidebarMenuButton isActive={isActive('/runs')}>
                <TimerIcon /> Runs
              </SidebarMenuButton>
            </Link>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <Link to="/projects" onClick={closeMobileSidebar}>
              <SidebarMenuButton isActive={isActive('/projects')}>
                <BotIcon /> Projects
              </SidebarMenuButton>
            </Link>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <Link to="/employees" onClick={closeMobileSidebar}>
              <SidebarMenuButton isActive={isActive('/employees')}>
                <UserIcon /> Employees
              </SidebarMenuButton>
            </Link>
          </SidebarMenuItem>
        </SidebarHeader>

        <SidebarContent className="pt-2">
          <SidebarGroup className="hidden group-data-[collapsible=icon]:flex">
            <SidebarMenuButton
              className="text-primary"
              variant="outline"
              onClick={() => {
                issueFormViewmodel.open()
                closeMobileSidebar()
              }}
            >
              <PenLine /> New Issue
            </SidebarMenuButton>
          </SidebarGroup>

          <SidebarGroup className="hidden group-data-[collapsible=icon]:flex">
            <Link to="/" onClick={closeMobileSidebar}>
              <SidebarMenuButton isActive={isActive('/')}>
                <HomeIcon /> Dashboard
              </SidebarMenuButton>
            </Link>
          </SidebarGroup>

          <SidebarGroup className="hidden group-data-[collapsible=icon]:flex">
            <Link to="/my-work" onClick={closeMobileSidebar}>
              <SidebarMenuButton isActive={isActive('/my-work')}>
                <ListTodoIcon /> My Work
              </SidebarMenuButton>
            </Link>
          </SidebarGroup>

          <SidebarGroup className="hidden group-data-[collapsible=icon]:flex">
            <Link to="/issues" onClick={closeMobileSidebar}>
              <SidebarMenuButton isActive={isActive('/issues')}>
                <KanbanIcon /> Issues
              </SidebarMenuButton>
            </Link>
          </SidebarGroup>

          <SidebarGroup className="hidden group-data-[collapsible=icon]:flex">
            <Link to="/runs" onClick={closeMobileSidebar}>
              <SidebarMenuButton isActive={isActive('/runs')}>
                <TimerIcon /> Runs
              </SidebarMenuButton>
            </Link>
          </SidebarGroup>

          <SidebarGroup className="hidden group-data-[collapsible=icon]:flex">
            <Link to="/projects" onClick={closeMobileSidebar}>
              <SidebarMenuButton isActive={isActive('/projects')}>
                <BotIcon /> Projects
              </SidebarMenuButton>
            </Link>
          </SidebarGroup>

          <SidebarGroup className="hidden group-data-[collapsible=icon]:flex">
            <Link to="/employees" onClick={closeMobileSidebar}>
              <SidebarMenuButton isActive={isActive('/employees')}>
                <UserIcon /> Employees
              </SidebarMenuButton>
            </Link>
          </SidebarGroup>

          <SidebarGroup>
            <SidebarGroupLabel>
              <Link to="/runs" onClick={closeMobileSidebar}>
                Runs
              </Link>
            </SidebarGroupLabel>

            <SidebarGroupContent>
              {open &&
                appViewmodel.runs.runningRuns.map((run) => (
                  <SidebarMenuItem key={run.id}>
                    <Link params={{ runId: run.id }} to="/runs/$runId" onClick={closeMobileSidebar}>
                      <SidebarMenuButton isActive={isActive(`/runs/${run.id}`)}>
                        <div className="flex items-center gap-2 justify-between w-full">
                          <div className="truncate text-sm font-medium">
                            {AppModel.instance.resolveEmployeeName(run.employeeId) ?? 'Unknown employee'}
                          </div>

                          <span className="text-xs text-muted-foreground font-light">
                            {formatRunTime(run.startedAt ?? run.createdAt)}
                          </span>
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
            <SidebarGroupAction
              aria-label="Add project"
              onClick={() => {
                projectFormViewmodel.open()
                closeMobileSidebar()
              }}
            >
              <PlusIcon />
            </SidebarGroupAction>

            <SidebarGroupContent>
              {open &&
                AppModel.instance.projects.map((project) => (
                  <SidebarMenuItem key={project.id}>
                    <Link params={{ projectId: project.id }} to="/projects/$projectId" onClick={closeMobileSidebar}>
                      <SidebarMenuButton isActive={isActive(`/projects/${project.id}`)}>
                        {project.name}
                      </SidebarMenuButton>
                    </Link>
                  </SidebarMenuItem>
                ))}
            </SidebarGroupContent>
          </SidebarGroup>

          <SidebarGroup>
            <SidebarGroupLabel>Employees</SidebarGroupLabel>
            <SidebarGroupAction
              aria-label="Add employee"
              onClick={() => {
                employeeFormViewmodel.open()
                closeMobileSidebar()
              }}
            >
              <PlusIcon />
            </SidebarGroupAction>

            <SidebarGroupContent>
              {open &&
                AppModel.instance.employees.map((employee) => {
                  const Icon = employeeIconValueToIcon(employee.icon!)

                  return (
                    <SidebarMenuItem key={employee.id}>
                      <Link
                        params={{ employeeId: employee.id }}
                        to="/employees/$employeeId"
                        onClick={closeMobileSidebar}
                      >
                        <SidebarMenuButton
                          className="[&_svg]:text-inherit data-active:[&_svg]:text-inherit"
                          isActive={isActive(`/employees/${employee.id}`)}
                        >
                          <TextColoredSpan color={employee.color as ColorVariant}>
                            <Icon className="text-inherit" />
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
        <SidebarFooter className="border-t border-sidebar-border/70">
          <Button
            aria-label="Log out"
            className="w-full justify-start group-data-[collapsible=icon]:size-9 group-data-[collapsible=icon]:justify-center"
            type="button"
            variant="ghost"
            onClick={() => void handleLogout()}
          >
            <LogOutIcon />
            {open && 'Log out'}
          </Button>
          <Link to="/settings" onClick={closeMobileSidebar}>
            <SidebarMenuButton isActive={isActive('/settings')}>
              <SlidersHorizontalIcon />
              {open && 'Settings'}
            </SidebarMenuButton>
          </Link>
          {isDev && (
            <Button
              aria-label="Nuke database"
              className="w-full justify-start group-data-[collapsible=icon]:size-9 group-data-[collapsible=icon]:justify-center"
              disabled={isNukingDatabase}
              type="button"
              variant="destructive-outline"
              onClick={() => setIsNukeDialogOpen(true)}
            >
              <Trash2Icon />
              {open && (isNukingDatabase ? 'Nuking database...' : 'Nuke Database')}
            </Button>
          )}
        </SidebarFooter>
        <SidebarRail />
      </Sidebar>
      <ConfirmationDialog
        cancelLabel="Keep database"
        confirmLabel="Nuke database"
        description="This removes the local database and sends you back through onboarding."
        onConfirm={() => void handleNukeDatabase()}
        onOpenChange={setIsNukeDialogOpen}
        open={isNukeDialogOpen}
        title="Nuke local database?"
      />
      <IssueForm viewmodel={issueFormViewmodel} />
      <ProjectForm viewmodel={projectFormViewmodel} />
      <EmployeeForm viewmodel={employeeFormViewmodel} />
    </>
  )
})
