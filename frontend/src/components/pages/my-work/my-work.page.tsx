import { InboxIcon, MessageSquareTextIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { Page } from '@/components/layouts/page'
import { Card, CardContent } from '@/components/ui/card'
import { MyWorkList } from './components/my-work-list'
import type { MyWorkViewmodel } from './my-work.viewmodel'

interface MyWorkPageProps {
  viewmodel: MyWorkViewmodel
}

export const MyWorkPage = observer(({ viewmodel }: MyWorkPageProps) => {
  return (
    <Page className="overflow-y-auto px-3 pb-6 md:px-5">
      <div className="mx-auto flex w-full max-w-7xl flex-col gap-5">
        <div className="space-y-1">
          <h1 className="text-2xl font-semibold tracking-tight">My Work</h1>
          <p className="text-sm text-muted-foreground">
            Your current queue for assignments and direct mentions, ranked by freshest relevant activity.
          </p>
        </div>

        {viewmodel.errorMessage ? (
          <Card>
            <CardContent className="py-4 text-sm text-destructive">{viewmodel.errorMessage}</CardContent>
          </Card>
        ) : null}

        <section className="space-y-3">
          <div className="flex items-center gap-2">
            <InboxIcon className="size-4 text-muted-foreground" />
            <h2 className="text-lg font-medium">Assigned to you</h2>
          </div>
          <MyWorkList
            emptyDescription="Assigned issues will appear here once work is routed to you."
            emptyTitle="Nothing assigned right now"
            items={viewmodel.assigned}
          />
        </section>

        <section className="space-y-3">
          <div className="flex items-center gap-2">
            <MessageSquareTextIcon className="size-4 text-muted-foreground" />
            <h2 className="text-lg font-medium">Mentioned</h2>
          </div>
          <MyWorkList
            emptyDescription="Direct @mentions on active issues will appear here for quick triage."
            emptyTitle="No recent mentions"
            items={viewmodel.mentioned}
          />
        </section>
      </div>
    </Page>
  )
})
