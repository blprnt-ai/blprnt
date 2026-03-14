import { debounce } from 'lodash'
import { reaction } from 'mobx'
import { useEffect, useRef } from 'react'
import { cn } from '@/lib/utils/cn'
import type { MessageStatus } from '@/types'
import { BucketRow } from './bucket-row'
import { useSubagentConversationViewmodel } from './subagent-conversation-viewmodel'

interface SubagentConversationProps {
  status: MessageStatus
}

const PADDING = 300

export const SubagentConversation = ({ status }: SubagentConversationProps) => {
  const containerRef = useRef<HTMLDivElement>(null)
  const viewmodel = useSubagentConversationViewmodel()

  // biome-ignore lint/correctness/useExhaustiveDependencies: mobx
  useEffect(() => {
    if (!containerRef.current) return

    const checkIfAtBottom = debounce(() => {
      if (containerRef.current) {
        const scrollTop = containerRef.current.scrollTop + containerRef.current.clientHeight + PADDING
        viewmodel.setScrollTop(scrollTop)
        const isAtBottom = scrollTop >= containerRef.current.scrollHeight

        viewmodel.setIsAtBottom(isAtBottom)
      }
    }, 300)

    containerRef.current.addEventListener('scroll', checkIfAtBottom)

    const ubsub = reaction(
      () => [viewmodel.isAtBottom, viewmodel.messages.size],
      ([isAtBottom]) => {
        if (!isAtBottom) return

        const scrollTop = containerRef.current!.scrollHeight + PADDING
        containerRef.current!.scrollTo({ behavior: 'smooth', top: scrollTop })
      },
    )

    return () => {
      ubsub()
      containerRef.current?.removeEventListener('scroll', checkIfAtBottom)
    }
  }, [])

  if (!viewmodel.sessionId) {
    return <div className="text-xs text-muted-foreground">Subagent session unavailable.</div>
  }

  if (!viewmodel.buckets.length) {
    return <div className="text-xs text-muted-foreground">No subagent output yet.</div>
  }

  return (
    <div
      ref={containerRef}
      className={cn(
        'max-h-[400px] overflow-y-auto overflow-x-hidden rounded-md border border-dashed border-border p-2',
        status === 'in_progress' && 'bg-primary/10',
        status === 'completed' && 'bg-green-950/20',
        status === 'error' && 'bg-red-950/20',
        status === 'pending' && 'bg-yellow-400/10',
      )}
    >
      <div className="flex flex-col gap-2 h-full">
        {viewmodel.buckets.map((bucket, index) => (
          <BucketRow
            key={`${bucket.type}-${index}`}
            bucket={bucket}
            isLast={index === viewmodel.buckets.length - 1}
            messages={viewmodel.messages}
          />
        ))}
      </div>
    </div>
  )
}
