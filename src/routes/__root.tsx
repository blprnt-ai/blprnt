import { createRootRoute, Outlet, useRouterState } from '@tanstack/react-router'
import { Loader2Icon } from 'lucide-react'
import { AnimatePresence, motion } from 'motion/react'
import { AppSidebar } from '@/components/organisms/app-sidebar'
import { Header } from '@/components/organisms/header'
import { SidebarProvider } from '@/components/ui/sidebar'
import { useAppModel } from '@/hooks/use-app-model'

const RootLayout = () => {
  const appModel = useAppModel()

  return (
    <AnimatePresence mode="wait">
      <motion.div
        key={appModel.isLoading ? 'loading' : 'content'}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        initial={{ opacity: 0 }}
        transition={{ duration: 0.2 }}
      >
        {appModel.isLoading ? <AppLoader /> : <AppContent />}
      </motion.div>
    </AnimatePresence>
  )
}

const AppLoader = () => {
  return (
    <div className="flex h-screen w-screen items-center justify-center gap-2">
      <div>
        <Loader2Icon className="size-4 animate-spin text-cyan-400" />
      </div>
      <div>Loading...</div>
    </div>
  )
}

const AppContent = () => {
  const route = useRouterState()

  return (
    <SidebarProvider>
      <AppSidebar />
      <main className="w-full">
        <Header />
        <AnimatePresence mode="wait">
          <motion.div key={route.location.pathname} animate={{ opacity: 1 }} initial={{ opacity: 0 }}>
            <Outlet />
          </motion.div>
        </AnimatePresence>
      </main>
    </SidebarProvider>
  )
}

export const Route = createRootRoute({ component: RootLayout })
