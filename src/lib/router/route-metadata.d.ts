declare module '@tanstack/router-core' {
  interface StaticDataRouteOption {
    breadcrumb?: string | ((params: Record<string, string>) => string)
  }
}

export {}
