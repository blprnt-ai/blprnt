import type { PropsWithChildren } from 'react'
import { Badge } from '@/components/atoms/badge'
import { CodeInlineOrBlock } from '@/components/atoms/code'

interface KvpProps extends PropsWithChildren {
  k: string
  badge?: string
}

export const Kvp = (props: KvpProps) => {
  const { k, badge, children } = props
  return (
    <div className="grid grid-cols-1 gap-1 rounded-md border p-3 sm:grid-cols-3">
      <div className="flex items-center gap-2">
        <span className="font-medium">{k}</span>
        {badge ? <Badge variant="secondary">{badge}</Badge> : null}
      </div>
      <div className="sm:col-span-2">
        <CodeInlineOrBlock>{children}</CodeInlineOrBlock>
      </div>
    </div>
  )
}
