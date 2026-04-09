import { BellDotIcon, InboxIcon, ListTodoIcon } from 'lucide-react'
import type { MyWorkViewmodel } from '../my-work.viewmodel'

interface MyWorkOverviewProps {
  viewmodel: MyWorkViewmodel
}

export const MyWorkOverview = ({ viewmodel }: MyWorkOverviewProps) => {
  const stats = [
    {
      icon: ListTodoIcon,
      label: 'Total queue',
      value: viewmodel.totalItems,
    },
    {
      icon: InboxIcon,
      label: 'Assigned',
      value: viewmodel.assigned.length,
    },
    {
      icon: BellDotIcon,
      label: 'Mentions',
      value: viewmodel.mentioned.length,
    },
  ]

  return (
    <div className="grid gap-3 sm:grid-cols-3">
      {stats.map((stat) => {
        const Icon = stat.icon

        return (
          <div key={stat.label} className="rounded-sm border border-border/60 bg-background/80 px-4 py-4">
            <div className="flex items-center justify-between gap-3">
              <div>
                <div className="text-xs uppercase tracking-[0.16em] text-muted-foreground">{stat.label}</div>
                <div className="mt-2 text-2xl font-semibold tracking-tight text-foreground">{stat.value}</div>
              </div>
              <div className="flex size-10 items-center justify-center rounded-full border border-border/60 bg-muted/30">
                <Icon className="size-4 text-muted-foreground" />
              </div>
            </div>
          </div>
        )
      })}
    </div>
  )
}
