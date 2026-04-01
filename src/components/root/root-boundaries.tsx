import type { ErrorComponentProps } from '@tanstack/react-router'
import { Link } from '@tanstack/react-router'
import { AlertTriangleIcon, RefreshCcwIcon } from 'lucide-react'
import { Button, buttonVariants } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card'

interface BootstrapErrorScreenProps {
  error: unknown
  title?: string
  description?: string
}

export const BootstrapErrorScreen = ({
  error,
  title = 'Unable to start blprnt',
  description = 'The app could not finish loading its local data. Check the error below and try again.',
}: BootstrapErrorScreenProps) => {
  const message = error instanceof Error ? error.message : 'Unknown startup error'

  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-6">
      <Card className="w-full max-w-xl">
        <CardHeader>
          <div className="flex items-center gap-3">
            <div className="flex size-10 items-center justify-center rounded-full bg-destructive/10 text-destructive">
              <AlertTriangleIcon className="size-5" />
            </div>
            <div>
              <CardTitle>{title}</CardTitle>
              <CardDescription>{description}</CardDescription>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <pre className="overflow-x-auto rounded-sm border border-border/60 bg-muted/40 p-4 text-sm text-muted-foreground">
            {message}
          </pre>
        </CardContent>
        <CardFooter>
          <Button className="gap-2" onClick={() => window.location.reload()}>
            <RefreshCcwIcon className="size-4" />
            Retry
          </Button>
        </CardFooter>
      </Card>
    </div>
  )
}

export const RootErrorBoundary = ({ error }: ErrorComponentProps) => {
  return (
    <BootstrapErrorScreen
      description="A route failed while loading or rendering. Reload first; if it persists, the app state or local database needs attention."
      error={error}
      title="Something went wrong"
    />
  )
}

export const RootNotFound = () => {
  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-6">
      <Card className="w-full max-w-lg">
        <CardHeader>
          <CardTitle>Page not found</CardTitle>
          <CardDescription>The route you opened does not exist in this local app.</CardDescription>
        </CardHeader>
        <CardFooter className="gap-3">
          <Link className={buttonVariants()} to="/">
            Go home
          </Link>
          <Button variant="outline" onClick={() => window.history.back()}>
            Go back
          </Button>
        </CardFooter>
      </Card>
    </div>
  )
}
