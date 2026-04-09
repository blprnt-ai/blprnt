import { createRootRoute } from '@tanstack/react-router'
import { ProductShell } from '@/components/layouts/product-shell'
import { RootErrorBoundary, RootNotFound } from '@/components/root/root-boundaries'

export const Route = createRootRoute({
  component: ProductShell,
  errorComponent: RootErrorBoundary,
  notFoundComponent: RootNotFound,
})
