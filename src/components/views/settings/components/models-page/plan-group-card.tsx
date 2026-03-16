import type { LlmModelResponse } from '@/bindings'
import { cn } from '@/lib/utils/cn'

interface PlanGroupCardProps {
  title: string
  models: LlmModelResponse[] | undefined
  isPremium?: boolean
}

export const PlanGroupCard = ({ isPremium, title, models }: PlanGroupCardProps) => {
  if (!models) return null

  return (
    <div className="mb-4">
      <div className={cn('text-md font-semibold', isPremium && 'text-primary')}>{title}</div>

      {models.map((model) => (
        <div
          key={model.slug}
          className={cn('px-2 py-1 text-xs text-muted-foreground/60', model.auto_router && 'text-accent-foreground')}
        >
          {model.name}
        </div>
      ))}
    </div>
  )
}
