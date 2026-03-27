import { Link, useRouterState } from '@tanstack/react-router'
import { BotIcon, HomeIcon, KanbanIcon, PenLine, PlusIcon, TimerIcon, UserIcon } from 'lucide-react'
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
import { AppModel } from '@/models/app.model'
import { employeeIconValueToIcon } from '../ui/employee-label'

export const AppSidebar = () => {
  const pathname = useRouterState({ select: (state) => state.location.pathname })
  const { open } = useSidebar()

  const isActive = (path: string) => pathname === path

  return (
    <Sidebar collapsible="icon" variant="floating">
      <SidebarHeader className="list-none group-data-[collapsible=icon]:hidden">
        <SidebarMenuItem>
          <SidebarMenuButton className="text-primary" variant="outline">
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
      </SidebarHeader>

      <SidebarContent>
        <SidebarGroup className="hidden group-data-[collapsible=icon]:flex">
          <SidebarMenuButton className="text-primary" variant="outline">
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

        <SidebarGroup>
          <SidebarGroupLabel>Runs</SidebarGroupLabel>
          <SidebarGroupAction>
            <PlusIcon />
          </SidebarGroupAction>

          <SidebarGroupContent>
            {!open && (
              <SidebarMenuItem>
                <SidebarMenuButton>
                  <TimerIcon />
                  Runs
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
                <SidebarMenuButton>
                  <BotIcon />
                  Projects
                </SidebarMenuButton>
              </SidebarMenuItem>
            )}
            {open &&
              AppModel.instance.projects.map((project) => (
                <SidebarMenuItem key={project.id}>
                  <SidebarMenuButton>{project.name}</SidebarMenuButton>
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
            {!open && (
              <SidebarMenuItem>
                <SidebarMenuButton>
                  <UserIcon />
                  Employees
                </SidebarMenuButton>
              </SidebarMenuItem>
            )}
            {open &&
              AppModel.instance.employees.map((employee) => {
                const Icon = employeeIconValueToIcon(employee.icon!)

                return (
                  <SidebarMenuItem key={employee.id}>
                    <SidebarMenuButton>
                      <Icon />
                      {employee.name}
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                )
              })}
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter />
      <SidebarRail />
    </Sidebar>
  )
}
