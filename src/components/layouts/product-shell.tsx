import { Navigate, Outlet, useRouterState } from '@tanstack/react-router'
import { AnimatePresence } from 'motion/react'
import { useMemo } from 'react'

import { useAppViewmodel } from '@/app.viewmodel'
import { AppSidebar } from '@/components/organisms/app-sidebar'
import { Header } from '@/components/organisms/header'
import { SidebarProvider } from '@/components/ui/sidebar'
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
        hasOwner: appViewmodel.hasOwner,
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
  return (
    <>
      {showProductShell && <AppSidebar />}
      <main className={cn('min-w-0 flex-1 overflow-x-hidden', showProductShell && 'flex min-h-svh flex-col')}>
        {showProductShell && <Header />}
        <AnimatePresence mode="wait">
          <Outlet />
        </AnimatePresence>
      </main>
    </>
  )
}
