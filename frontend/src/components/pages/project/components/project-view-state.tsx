import type * as React from 'react'
import { Card, CardContent } from '@/components/ui/card'

export const ProjectViewState = ({
  action,
  icon: Icon,
  message,
  title,
  minHeight = 'min-h-[320px]',
}: {
  action?: React.ReactNode
  icon: React.ComponentType<{ className?: string }>
  message: string
  title: string
  minHeight?: string
}) => {
  return (
    <Card className="border-border/60">
      <CardContent className={`flex ${minHeight} flex-col items-center justify-center gap-3 px-6 py-10 text-center`}>
        <div className="flex size-12 items-center justify-center rounded-full border border-border/60 bg-muted/30">
          <Icon className="size-5 text-muted-foreground" />
        </div>
        <div className="space-y-1">
          <h3 className="text-base font-medium">{title}</h3>
          <p className="text-sm text-muted-foreground">{message}</p>
        </div>
        {action}
      </CardContent>
    </Card>
  )
}
