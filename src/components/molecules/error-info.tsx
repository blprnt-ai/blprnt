import { BgGrid } from '@/components/atoms/bg-grid'
import { Button } from '@/components/atoms/button'

interface ErrorInfoProps {
  title: string
  error: React.ReactNode
  action?: () => void
  actionLabel?: string
}
export const ErrorInfo = ({ title, error, action, actionLabel }: ErrorInfoProps) => {
  return (
    <>
      <BgGrid />
      <div className="flex flex-col items-center justify-center h-full z-100 relative">
        <div className="flex flex-col items-center justify-center gap-2 border border-destructive border-dashed rounded-md px-12 py-8 bg-destructive-background bg-gradient-glow-dark">
          <div className="text-2xl font-medium">{title}</div>
          <div className="text-lg text-muted-foreground">{error}</div>

          {action && (
            <Button variant="ghost" onClick={action}>
              {actionLabel ?? 'Close Tab'}
            </Button>
          )}
        </div>
      </div>
    </>
  )
}
