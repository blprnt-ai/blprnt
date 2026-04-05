type BreadcrumbValue = string | ((params: Record<string, string>) => string)

interface BreadcrumbRoute {
  options: {
    staticData?: {
      breadcrumb?: BreadcrumbValue
    }
  }
  path: string
  to: string
}

interface BuildBreadcrumbsOptions {
  currentParams: Record<string, string>
  currentRouteId: string
  getOverrideLabel: (routeId: string) => string | undefined
  routesById: Record<string, unknown>
}

interface ResolveBreadcrumbLabelOptions {
  overrideLabel?: string
  params: Record<string, string>
  route: BreadcrumbRoute
}

export interface BreadcrumbCrumb {
  href: string
  label: string
}

export const buildBreadcrumbs = ({
  currentParams,
  currentRouteId,
  getOverrideLabel,
  routesById,
}: BuildBreadcrumbsOptions): BreadcrumbCrumb[] => {
  return getBreadcrumbRouteIds(currentRouteId).flatMap((routeId) => {
    const route = getBreadcrumbRoute(routesById, routeId)
    if (!route) return []

    return [
      {
        href: route.to,
        label: resolveBreadcrumbLabel({
          overrideLabel: getOverrideLabel(routeId),
          params: currentParams,
          route: route,
        }),
      },
    ]
  })
}

const resolveBreadcrumbLabel = ({ overrideLabel, params, route }: ResolveBreadcrumbLabelOptions) => {
  if (overrideLabel) return overrideLabel

  const breadcrumb = route.options.staticData?.breadcrumb

  if (typeof breadcrumb === 'function') {
    return breadcrumb(params)
  }

  return breadcrumb ?? route.path
}

const getBreadcrumbRouteIds = (routeId: string) => {
  if (routeId === '/') return ['/']

  const segments = routeId.split('/').filter(Boolean)
  const routeIds: string[] = []

  for (let index = 0; index < segments.length; index += 1) {
    routeIds.push(`/${segments.slice(0, index + 1).join('/')}/`)
  }

  return routeIds
}

const getBreadcrumbRoute = (routesById: Record<string, unknown>, routeId: string) => {
  return (routesById[routeId] ?? routesById[routeId.replace(/\/$/, '')] ?? routesById[`${routeId}/`]) as
    | BreadcrumbRoute
    | undefined
}
