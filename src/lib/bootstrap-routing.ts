type BootstrapRouteState = {
  hasOwner?: boolean
  isLoginConfigured?: boolean
  pathname: string
  isOnboarded: boolean
  isAuthenticated: boolean
}

const AUTH_FREE_ROUTES = new Set(['/login', '/bootstrap', '/onboarding'])

export const shouldRenderProductShell = (pathname: string) => !AUTH_FREE_ROUTES.has(pathname)

export const getBootstrapRedirectPath = ({
  hasOwner = true,
  isLoginConfigured = true,
  pathname,
  isOnboarded,
  isAuthenticated,
}: BootstrapRouteState) => {
  if (!isAuthenticated) {
    if (!hasOwner || !isLoginConfigured) {
      return pathname === '/bootstrap' ? null : '/bootstrap'
    }

    return pathname === '/login' ? null : '/login'
  }

  if (!isOnboarded && pathname === '/') {
    return '/onboarding'
  }

  if (!isOnboarded && pathname === '/login') {
    return '/onboarding'
  }

  if (!isOnboarded && pathname === '/bootstrap') {
    return '/onboarding'
  }

  if (isOnboarded && AUTH_FREE_ROUTES.has(pathname)) {
    return '/'
  }

  return null
}
