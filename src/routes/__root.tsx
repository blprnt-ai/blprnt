import { createRootRoute, Outlet } from '@tanstack/react-router'
import { AppSidebar } from '@/components/organisms/app-sidebar'
import { Header } from '@/components/organisms/header'
import { SidebarProvider } from '@/components/ui/sidebar'

const RootLayout = () => (
  <SidebarProvider>
    <AppSidebar />
    <main className="w-full">
      <Header />
      <Outlet />
    </main>
  </SidebarProvider>
)

export const Route = createRootRoute({ component: RootLayout })
