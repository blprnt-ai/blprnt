import { MessageBucketType } from '@/lib/utils/message-utils'
import { ChainBucket } from './chain-bucket'
import { ConversationBucket } from './conversation-bucket'
import { QuestionsBucket } from './questions-bucket'
import { SubAgentBucket } from './subagent-bucket'

export const BucketRow = ({
  bucket,
  isLast,
}: {
  bucket: { type: MessageBucketType; messageKeys: string[] }
  isLast: boolean
}) => {
  switch (bucket.type) {
    case MessageBucketType.Conversation:
      return <ConversationBucket messageKeys={bucket.messageKeys} />
    case MessageBucketType.ChainOfThought:
      return <ChainBucket isLast={isLast} messageKeys={bucket.messageKeys} />
    case MessageBucketType.SubAgent:
      return <SubAgentBucket messageKeys={bucket.messageKeys} />
    case MessageBucketType.QuestionAnswer:
      return <QuestionsBucket messageKeys={bucket.messageKeys} />
    default:
      return null
  }
}
