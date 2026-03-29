import { Navigate, Outlet, useRouterState } from '@tanstack/react-router'
import { AnimatePresence } from 'motion/react'
import { useMemo } from 'react'

import { useAppViewmodel } from '@/app.viewmodel'
import { AppSidebar } from '@/components/organisms/app-sidebar'
import { Header } from '@/components/organisms/header'
import { SidebarProvider, useSidebar } from '@/components/ui/sidebar'
import { getBootstrapRedirectPath, shouldRenderProductShell } from '@/lib/bootstrap-routing'
import { cn } from '@/lib/utils'

export const ProductShell = () => {
  const appViewmodel = useAppViewmodel()

  const pathname = useRouterState({
    select: (state) => state.location.pathname,
  })

  // biome-ignore lint/correctness/useExhaustiveDependencies: mobx
  const redirectPath = useMemo(
    () =>
      getBootstrapRedirectPath({
        isOnboarded: appViewmodel.isOnboarded,
        pathname,
      }),
    [pathname],
  )
  const showProductShell = useMemo(() => shouldRenderProductShell(pathname), [pathname])

  if (redirectPath) return <Navigate replace to={redirectPath} />

  return (
    <SidebarProvider>
      <MainContent showProductShell={showProductShell} />
    </SidebarProvider>
  )
}

interface MainContentProps {
  showProductShell: boolean
}

const MainContent = ({ showProductShell }: MainContentProps) => {
  const sidebar = useSidebar()

  return (
    <>
      {showProductShell && <AppSidebar />}
      <main
        className={cn(
          sidebar.isMobile || !showProductShell && 'w-full',
          showProductShell && !sidebar.isMobile && sidebar.open && 'w-[calc(100%-16rem)]',
          showProductShell && !sidebar.isMobile && !sidebar.open && 'w-[calc(100%-4rem)]',
        )}
      >
        {showProductShell && <Header />}
        <AnimatePresence mode="wait">
          <Outlet />
        </AnimatePresence>
      </main>
    </>
  )
}
