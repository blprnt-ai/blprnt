import { InboxIcon, MessageSquareTextIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { Page } from '@/components/layouts/page'
import { Card, CardContent } from '@/components/ui/card'
import { MyWorkList } from './components/my-work-list'
import { MyWorkOverview } from './components/my-work-overview'
import { MyWorkSection } from './components/my-work-section'
import type { MyWorkViewmodel } from './my-work.viewmodel'

interface MyWorkPageProps {
  viewmodel: MyWorkViewmodel
}

export const MyWorkPage = observer(({ viewmodel }: MyWorkPageProps) => {
  return (
    <Page className="overflow-y-auto px-3 pb-8 md:px-5">
      <div className="mx-auto flex w-full max-w-7xl flex-col gap-6">
        <div className="grid gap-4 xl:grid-cols-[minmax(0,1.4fr)_minmax(280px,0.9fr)]">
          <Card className="border-border/60 bg-linear-to-br from-background via-background to-muted/25">
            <CardContent className="flex flex-col gap-4 py-6">
              <div className="space-y-2">
                <div className="text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">
                  Personal queue
                </div>
                <div className="space-y-1">
                  <h1 className="text-3xl font-semibold tracking-tight text-foreground">My Work</h1>
                  <p className="max-w-2xl text-sm text-muted-foreground">
                    Active assignments and direct mentions, ordered by the latest activity that needs your attention.
                  </p>
                </div>
              </div>

              <MyWorkOverview viewmodel={viewmodel} />
            </CardContent>
          </Card>

          <Card className="border-border/60 bg-muted/20">
            <CardContent className="flex h-full flex-col justify-between gap-4 py-6">
              <div className="space-y-1">
                <div className="text-sm font-medium">Focus next</div>
                <p className="text-sm text-muted-foreground">
                  {viewmodel.newestItem
                    ? `${viewmodel.newestItem.issue_identifier} was updated most recently and currently leads your queue.`
                    : 'Your queue is clear right now.'}
                </p>
              </div>

              {viewmodel.newestItem ? (
                <div className="rounded-sm border border-border/60 bg-background/80 px-4 py-3">
                  <div className="text-sm font-medium text-foreground">{viewmodel.newestItem.title}</div>
                  <div className="mt-1 text-xs uppercase tracking-[0.16em] text-muted-foreground">
                    {viewmodel.newestItem.reason === 'assigned' ? 'Assigned' : 'Mentioned'}
                  </div>
                </div>
              ) : null}
            </CardContent>
          </Card>
        </div>

        {viewmodel.errorMessage ? (
          <Card>
            <CardContent className="py-4 text-sm text-destructive">{viewmodel.errorMessage}</CardContent>
          </Card>
        ) : null}

        <MyWorkSection
          count={viewmodel.assigned.length}
          description="Issues currently routed to you."
          icon={InboxIcon}
          title="Assigned to you"
        >
          <MyWorkList
            emptyDescription="Assigned issues will appear here once work is routed to you."
            emptyTitle="Nothing assigned right now"
            items={viewmodel.assigned}
          />
        </MyWorkSection>

        <MyWorkSection
          count={viewmodel.mentioned.length}
          description="Direct mentions that still need a pass from you."
          icon={MessageSquareTextIcon}
          title="Mentioned"
        >
          <MyWorkList
            emptyDescription="Direct @mentions on active issues will appear here for quick triage."
            emptyTitle="No recent mentions"
            items={viewmodel.mentioned}
          />
        </MyWorkSection>
      </div>
    </Page>
  )
})
