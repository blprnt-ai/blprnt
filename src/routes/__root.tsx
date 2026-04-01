import { createRootRoute } from '@tanstack/react-router'

import { RootErrorBoundary, RootNotFound } from '@/components/root/root-boundaries'
import { ProductShell } from '@/components/layouts/product-shell'

export const Route = createRootRoute({
  component: ProductShell,
  errorComponent: RootErrorBoundary,
  notFoundComponent: RootNotFound,
})
