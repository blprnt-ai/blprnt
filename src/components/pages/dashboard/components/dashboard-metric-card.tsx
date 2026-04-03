import type { LucideIcon } from 'lucide-react'
import { Card, CardContent } from '@/components/ui/card'
import { cn } from '@/lib/utils'

interface DashboardMetricCardProps {
  icon?: LucideIcon
  label: string
  value: number | string
  helper?: string
  tone?: 'default' | 'dark'
}

export const DashboardMetricCard = ({ icon: Icon, label, value, helper, tone = 'default' }: DashboardMetricCardProps) => {
  return (
    <Card className={cn(tone === 'dark' && 'border-white/10 bg-white/5 text-white shadow-none')}>
      <CardContent className="space-y-3 p-5">
        <div className="flex items-center justify-between gap-3">
          <p className={cn('text-xs uppercase tracking-[0.22em] text-muted-foreground', tone === 'dark' && 'text-slate-300')}>{label}</p>
          {Icon ? <Icon className={cn('size-4 text-cyan-400', tone === 'dark' && 'text-cyan-300')} /> : null}
        </div>
        <div className="space-y-1">
          <p className="text-3xl font-semibold tracking-tight">{value}</p>
          {helper ? <p className={cn('text-sm text-muted-foreground', tone === 'dark' && 'text-slate-300/90')}>{helper}</p> : null}
        </div>
      </CardContent>
    </Card>
  )
}