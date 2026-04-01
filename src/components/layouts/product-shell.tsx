import { Navigate, Outlet, useRouterState } from '@tanstack/react-router'
import { observer } from 'mobx-react-lite'
import { AnimatePresence } from 'motion/react'
import { useMemo } from 'react'

import { useAppViewmodel } from '@/app.viewmodel'
import { AppSidebar } from '@/components/organisms/app-sidebar'
import { Header } from '@/components/organisms/header'
import { FloatingDock } from '@/components/ui/floating-dock'
import { SidebarProvider } from '@/components/ui/sidebar'
import { getBootstrapRedirectPath, shouldRenderProductShell } from '@/lib/bootstrap-routing'
import { cn } from '@/lib/utils'
import { BotIcon, HomeIcon, KanbanIcon, TimerIcon, UserIcon } from 'lucide-react'

export const ProductShell = observer(() => {
  const appViewmodel = useAppViewmodel()

  const pathname = useRouterState({
    select: (state) => state.location.pathname,
  })

  const redirectPath = getBootstrapRedirectPath({
    hasOwner: appViewmodel.hasOwner,
    isOnboarded: appViewmodel.isOnboarded,
    pathname,
  })
  const showProductShell = useMemo(() => shouldRenderProductShell(pathname), [pathname])

  if (redirectPath) return <Navigate replace to={redirectPath} />

  return (
    <SidebarProvider>
      <MainContent showProductShell={showProductShell} />
    </SidebarProvider>
  )
})

interface MainContentProps {
  showProductShell: boolean
}

const MainContent = ({ showProductShell }: MainContentProps) => {
  const pathname = useRouterState({
    select: (state) => state.location.pathname,
  })

  const dockItems = [
    { href: '/', icon: <HomeIcon className="size-4" />, isActive: pathname === '/', title: 'Dashboard' },
    {
      href: '/issues',
      icon: <KanbanIcon className="size-4" />,
      isActive: pathname.startsWith('/issues'),
      title: 'Issues',
    },
    { href: '/runs', icon: <TimerIcon className="size-4" />, isActive: pathname.startsWith('/runs'), title: 'Runs' },
    {
      href: '/projects',
      icon: <BotIcon className="size-4" />,
      isActive: pathname.startsWith('/projects'),
      title: 'Projects',
    },
    {
      href: '/employees',
      icon: <UserIcon className="size-4" />,
      isActive: pathname.startsWith('/employees'),
      title: 'Employees',
    },
  ]

  return (
    <>
      {showProductShell && <AppSidebar />}
      <main
        className={cn('min-w-0 flex-1 overflow-x-hidden', showProductShell && 'flex min-h-svh flex-col pb-20 md:pb-0')}
      >
        {showProductShell && <Header />}
        <AnimatePresence mode="wait">
          <Outlet />
        </AnimatePresence>
      </main>
      {showProductShell && <FloatingDock items={dockItems} mobileClassName="fixed right-4 bottom-4 z-40" />}
    </>
  )
}
