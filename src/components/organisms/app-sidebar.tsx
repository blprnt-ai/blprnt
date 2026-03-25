import { BotIcon, HomeIcon, PenLine, PlusIcon, TimerIcon, UserIcon } from 'lucide-react'
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

export const AppSidebar = () => {
  const { open } = useSidebar()

  return (
    <Sidebar collapsible="icon" variant="floating">
      <SidebarHeader className="list-none group-data-[collapsible=icon]:hidden">
        <SidebarMenuItem>
          <SidebarMenuButton className="text-primary" variant="outline">
            <PenLine /> New Issue
          </SidebarMenuButton>
        </SidebarMenuItem>
        <SidebarMenuItem>
          <SidebarMenuButton>
            <HomeIcon /> Dashboard
          </SidebarMenuButton>
        </SidebarMenuItem>
      </SidebarHeader>

      <SidebarContent>
        <SidebarGroup className="hidden group-data-[collapsible=icon]:flex">
          <SidebarMenuButton className="text-primary" variant="outline">
            <PenLine /> New Issue
          </SidebarMenuButton>
        </SidebarGroup>

        <SidebarGroup className="hidden group-data-[collapsible=icon]:flex">
          <SidebarMenuButton>
            <HomeIcon /> Dashboard 2
          </SidebarMenuButton>
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
          </SidebarGroupContent>
        </SidebarGroup>

        <SidebarGroup>
          <SidebarGroupLabel>Employees</SidebarGroupLabel>
          <SidebarGroupAction>
            <PlusIcon />
          </SidebarGroupAction>

          <SidebarGroupContent>
            <SidebarMenuItem className="group-data-[collapsible=icon]:opacity-100 opacity-0">
              <SidebarMenuButton>
                <UserIcon />
                Employees
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarGroupContent>
        </SidebarGroup>

        <SidebarGroup />

        <SidebarGroup>
          <SidebarGroupContent></SidebarGroupContent>
        </SidebarGroup>

        <SidebarGroup />
      </SidebarContent>
      <SidebarFooter />
      <SidebarRail />
    </Sidebar>
  )
}
