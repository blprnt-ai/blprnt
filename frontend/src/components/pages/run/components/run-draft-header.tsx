import { MessageSquareTextIcon } from 'lucide-react'
import { Card, CardContent } from '@/components/ui/card'

interface RunDraftHeaderProps {
  employeeName: string
}

export const RunDraftHeader = ({ employeeName }: RunDraftHeaderProps) => {
  return (
    <Card className="overflow-hidden border-border/60 bg-linear-to-br from-card via-card to-muted/30 py-0">
      <CardContent className="px-5 py-5 md:px-6">
        <div className="flex items-start gap-4">
          <div className="flex size-11 shrink-0 items-center justify-center rounded-xl border border-border/60 bg-background/75">
            <MessageSquareTextIcon className="size-5 text-muted-foreground" />
          </div>
          <div className="min-w-0 space-y-2">
            <div className="space-y-1">
              <h1 className="truncate text-xl font-medium tracking-tight">Conversation</h1>
              <p className="text-sm text-muted-foreground">{employeeName}</p>
            </div>
            <span className="rounded-full border border-border/60 bg-background/70 px-3 py-1 text-xs uppercase tracking-[0.18em] text-muted-foreground">
              Draft
            </span>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
