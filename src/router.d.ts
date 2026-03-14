import '@tanstack/react-router'
import type { RouteType } from './types'

declare module '@tanstack/react-router' {
  interface StaticDataRouteOption {
    /**
     * A friendly name for the route, used for navigation or display purposes.
     */
    displayName?: string
    type: RouteType
  }
}
