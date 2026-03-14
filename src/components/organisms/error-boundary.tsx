import { useState } from 'react'
import { ErrorBoundaryBasic } from '@/components/molecules/error-boundary-basic'
import { ErrorInfo } from '@/components/molecules/error-info'

interface ErrorBoundaryProps {
  children: React.ReactNode
  title?: string
  action?: () => void
  actionLabel?: string
  errorMessage?: React.ReactNode
}

export const ErrorBoundary = ({ children, title, action, actionLabel, errorMessage }: ErrorBoundaryProps) => {
  const [error, setError] = useState<React.ReactNode | null>(null)

  return (
    <ErrorBoundaryBasic
      fallback={
        <ErrorInfo
          action={action}
          actionLabel={actionLabel}
          error={error ?? 'Unknown error'}
          title={title ?? 'Error'}
        />
      }
      onError={(error) => {
        console.error(error)
        setError(errorMessage ?? error.message)
      }}
    >
      {children}
    </ErrorBoundaryBasic>
  )
}
