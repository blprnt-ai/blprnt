type ReactRootErrorPhase = 'caught' | 'recoverable' | 'uncaught'

export const reportError = (error: unknown, context?: string, details?: unknown) => {
  if (context && details !== undefined) {
    console.error('[blprnt]', context, error, details)
    return
  }

  if (context) {
    console.error('[blprnt]', context, error)
    return
  }

  console.error('[blprnt]', error)
}

export const createReactRootErrorHandler = (phase: ReactRootErrorPhase) => (error: unknown, errorInfo: unknown) => {
  reportError(error, `react root ${phase} error`, errorInfo)
}
