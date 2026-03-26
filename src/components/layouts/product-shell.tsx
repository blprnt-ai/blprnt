import { Navigate, Outlet, useRouterState } from '@tanstack/react-router'
import { AnimatePresence, motion } from 'motion/react'
import { useMemo } from 'react'
import { useAppViewmodel } from '@/app.viewmodel'
import { AppSidebar } from '@/components/organisms/app-sidebar'
import { Header } from '@/components/organisms/header'
import { SidebarProvider } from '@/components/ui/sidebar'
import { getBootstrapRedirectPath, shouldRenderProductShell } from '@/lib/bootstrap-routing'

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
      {showProductShell && <AppSidebar />}
      <main className="w-full">
        {showProductShell && <Header />}
        <AnimatePresence mode="wait">
          <motion.div key={pathname} animate={{ opacity: 1 }} initial={{ opacity: 0 }}>
            <Outlet />
          </motion.div>
        </AnimatePresence>
      </main>
    </SidebarProvider>
  )
}
