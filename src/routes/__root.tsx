import { createRootRoute } from '@tanstack/react-router'
import { AnimatePresence, motion } from 'motion/react'
import { useAppViewmodel } from '@/app.viewmodel'

import { ProductShell } from '@/components/layouts/product-shell'
import { AppLoader } from '@/components/organisms/app-loader'

const RootLayout = () => {
  const appViewmodel = useAppViewmodel()

  return (
    <AnimatePresence mode="wait">
      <motion.div
        key={appViewmodel.isLoading ? 'loading' : 'content'}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        initial={{ opacity: 0 }}
        transition={{ duration: 0.2 }}
      >
        {appViewmodel.isLoading ? <AppLoader /> : <ProductShell />}
      </motion.div>
    </AnimatePresence>
  )
}

export const Route = createRootRoute({ component: RootLayout })
