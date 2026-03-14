import { Empty, EmptyContent, EmptyHeader, EmptyTitle } from '@/components/atoms/empty'
import { MutedTextSmallItalic } from '@/components/atoms/muted-text'

export const EmptyOutputPanel = () => (
  <Empty className="pointer-events-none select-none">
    <EmptyHeader>
      <EmptyTitle>No messages yet.</EmptyTitle>
    </EmptyHeader>
    <EmptyContent>
      <MutedTextSmallItalic>Start a conversation by sending a prompt</MutedTextSmallItalic>
    </EmptyContent>
  </Empty>
)
