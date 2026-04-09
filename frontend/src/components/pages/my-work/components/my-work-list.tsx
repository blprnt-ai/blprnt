import { observer } from 'mobx-react-lite'
import type { MyWorkItemDto } from '@/bindings/MyWorkItemDto'
import { EmptyState } from '@/components/pages/issue/components/empty-state'
import { MyWorkRow } from './my-work-row'

interface MyWorkListProps {
  items: MyWorkItemDto[]
  emptyDescription: string
  emptyTitle: string
}

export const MyWorkList = observer(({ items, emptyDescription, emptyTitle }: MyWorkListProps) => {
  if (items.length === 0) {
    return <EmptyState description={emptyDescription} title={emptyTitle} />
  }

  return (
    <div className="space-y-3">
      {items.map((item) => (
        <MyWorkRow key={`${item.reason}-${item.issue_id}-${item.comment_id ?? 'issue'}`} item={item} />
      ))}
    </div>
  )
})
