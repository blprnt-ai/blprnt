type BootstrapRouteState = {
  pathname: string
  isOnboarded: boolean
}

export const shouldRenderProductShell = (pathname: string) => pathname !== '/onboarding'

export const getBootstrapRedirectPath = ({ pathname, isOnboarded }: BootstrapRouteState) => {
  if (!isOnboarded && pathname === '/') {
    return '/onboarding'
  }

  if (isOnboarded && pathname === '/onboarding') {
    return '/'
  }

  return null
}
