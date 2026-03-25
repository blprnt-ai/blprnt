import { Navigate, Outlet, useRouterState } from '@tanstack/react-router'
import { AnimatePresence, motion } from 'motion/react'
import { useAppViewmodel } from '@/app.viewmodel'
import { AppSidebar } from '@/components/organisms/app-sidebar'
import { Header } from '@/components/organisms/header'
import { SidebarProvider } from '@/components/ui/sidebar'
import { getBootstrapRedirectPath, shouldRenderProductShell } from '@/lib/bootstrap-routing'

export const ProductShell = () => {
  const pathname = useRouterState({
    select: (state) => state.location.pathname,
  })
  const appViewmodel = useAppViewmodel()
  const redirectPath = getBootstrapRedirectPath({
    isOnboarded: appViewmodel.isOnboarded,
    pathname,
  })
  const showProductShell = shouldRenderProductShell(pathname)

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
