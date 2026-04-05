import { Link, useMatches, useRouter } from '@tanstack/react-router'
import { observer } from 'mobx-react-lite'
import { Fragment } from 'react'
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from '@/components/ui/breadcrumb'
import { buildBreadcrumbs } from '@/lib/router/breadcrumbs'
import { HeaderBreadcrumbModel } from '@/models/header-breadcrumb.model'

export const HeaderBreadcrumbs = observer(() => {
  const router = useRouter()
  const matches = useMatches()
  const currentMatch = [...matches].reverse().find((match) => match.routeId !== '__root__')

  if (!currentMatch) return null

  const crumbs = buildBreadcrumbs({
    currentParams: currentMatch.params as Record<string, string>,
    currentRouteId: currentMatch.routeId,
    getOverrideLabel: (routeId) => HeaderBreadcrumbModel.instance.getLabel(routeId),
    routesById: router.looseRoutesById,
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
})
