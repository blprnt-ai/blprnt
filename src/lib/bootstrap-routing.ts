type BootstrapRouteState = {
  hasOwner?: boolean
  pathname: string
  isOnboarded: boolean
}

export const shouldRenderProductShell = (pathname: string) => pathname !== '/onboarding'

export const getBootstrapRedirectPath = ({ hasOwner = true, pathname, isOnboarded }: BootstrapRouteState) => {
  if (!hasOwner) {
    return pathname === '/onboarding' ? null : '/onboarding'
  }

  if (!isOnboarded && pathname === '/') {
    return '/onboarding'
  }

  if (isOnboarded && pathname === '/onboarding') {
    return '/'
  }

  return null
}
