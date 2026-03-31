import { Link, useMatches, useRouter } from '@tanstack/react-router'
import { Fragment } from 'react'
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from '@/components/ui/breadcrumb'

type BreadcrumbValue = string | ((params: Record<string, string>) => string)

export const HeaderBreadcrumbs = () => {
  const router = useRouter()
  const matches = useMatches()
  const currentMatch = [...matches].reverse().find((match) => match.routeId !== '__root__')

  if (!currentMatch) return null

  const crumbs = getBreadcrumbRouteIds(currentMatch.routeId).map((routeId) => {
    const route = router.looseRoutesById[routeId]
    const breadcrumb = (route.options.staticData as { breadcrumb?: BreadcrumbValue } | undefined)?.breadcrumb

    return {
      href: route.to,
      label:
        typeof breadcrumb === 'function'
          ? breadcrumb(currentMatch.params as Record<string, string>)
          : (breadcrumb ?? route.path),
    }
  })

  return (
    <Breadcrumb className="min-w-0">
      <BreadcrumbList className="flex-nowrap overflow-hidden whitespace-nowrap">
        {crumbs.map((crumb, index) => {
          const isLast = index === crumbs.length - 1

          return (
            <Fragment key={crumb.href}>
              <BreadcrumbItem className="min-w-0 shrink-0">
                {isLast ? (
                  <BreadcrumbPage className="truncate">{crumb.label}</BreadcrumbPage>
                ) : (
                  <BreadcrumbLink className="truncate" render={<Link to={crumb.href === '/' ? '/' : crumb.href} />}>
                    {crumb.label}
                  </BreadcrumbLink>
                )}
              </BreadcrumbItem>
              {!isLast ? <BreadcrumbSeparator /> : null}
            </Fragment>
          )
        })}
      </BreadcrumbList>
    </Breadcrumb>
  )
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
