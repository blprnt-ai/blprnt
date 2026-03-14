import type { MessageModel } from '@/lib/models/messages/message-factory'
import type { MessageBucket } from '@/lib/utils/message-utils'
import { MessageBucketType } from '@/lib/utils/message-utils'
import { ChainBucket } from './chain-bucket'
import { ConversationBucket } from './conversation-bucket'

export const BucketRow = ({
  bucket,
  isLast,
  messages,
}: {
  bucket: MessageBucket
  isLast: boolean
  messages: Map<string, MessageModel>
}) => {
  switch (bucket.type) {
    case MessageBucketType.Conversation:
      return <ConversationBucket messageKeys={bucket.messageKeys} messages={messages} />
    case MessageBucketType.ChainOfThought:
      return <ChainBucket isLast={isLast} messageKeys={bucket.messageKeys} messages={messages} />
    default:
      return null
  }
}
