import { ArrowUpIcon } from 'lucide-react'
import { Conversation, ConversationContent, ConversationScrollButton } from '@/components/ai-elements/conversation'
import { Button } from '@/components/atoms/button'
import { BucketRow } from '@/components/panels/session/organisms/session-conversation/bucket-row'
import { useSessionPanelViewmodel } from '@/components/panels/session/session-panel.viewmodel'

export const SessionConversation = () => {
  const viewmodel = useSessionPanelViewmodel()

  return (
    <Conversation>
      <ConversationContent className="py-2 flex flex-col gap-2 mr-1">
        {viewmodel.hasMoreBuckets && (
          <div className="flex justify-center">
            <Button
              className="rounded-full border-none bg-accent"
              size="icon"
              variant="outline"
              onClick={viewmodel.bumpBucketSize}
            >
              <ArrowUpIcon className="size-4" />
            </Button>
          </div>
        )}

        {viewmodel.visibleBuckets.map((bucket, index) => (
          <BucketRow
            key={`${bucket.type}-${index}`}
            bucket={bucket}
            isLast={index === viewmodel.visibleBuckets.length - 1}
          />
        ))}
      </ConversationContent>
      <ConversationScrollButton />
    </Conversation>
  )
}
