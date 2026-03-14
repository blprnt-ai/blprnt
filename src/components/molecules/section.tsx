import type { PropsWithChildren } from 'react'
import { Badge } from '@/components/atoms/badge'

interface SectionProps extends PropsWithChildren {
  title?: string
  badge?: string
}

export const Section = (props: SectionProps) => {
  const { title, badge, children } = props

  return (
    <section className="space-y-2">
      <div className="flex items-center gap-2">
        {title ? <h3 className="text-sm font-medium">{title}</h3> : null}
        {badge ? <Badge variant="outline">{badge}</Badge> : null}
      </div>
      <div className="space-y-2">{children}</div>
    </section>
  )
}
