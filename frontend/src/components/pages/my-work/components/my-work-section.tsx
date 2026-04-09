import type * as React from 'react'
import { IssueBadge } from '@/components/pages/issue/components/issue-badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'

interface MyWorkSectionProps {
  children: React.ReactNode
  count: number
  description: string
  icon: React.ComponentType<{ className?: string }>
  title: string
}

export const MyWorkSection = ({ children, count, description, icon: Icon, title }: MyWorkSectionProps) => {
  return (
    <section>
      <Card className="border-border/60 bg-background/85">
        <CardHeader className="gap-3 border-b">
          <div className="flex items-start justify-between gap-4">
            <div className="flex items-start gap-3">
              <div className="mt-0.5 flex size-10 items-center justify-center rounded-full border border-border/60 bg-muted/30">
                <Icon className="size-4 text-muted-foreground" />
              </div>
              <div className="space-y-1">
                <CardTitle>{title}</CardTitle>
                <p className="text-sm text-muted-foreground">{description}</p>
              </div>
            </div>

            <IssueBadge>{count}</IssueBadge>
          </div>
        </CardHeader>
        <CardContent className="py-6">{children}</CardContent>
      </Card>
    </section>
  )
}
