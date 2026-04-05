import { CardDescription, CardHeader, CardTitle } from '@/components/ui/card'

interface OnboardingCardHeaderProps {
  icon: React.ReactNode
  title: React.ReactNode
  subtitle: React.ReactNode
}

export const OnboardingCardHeader = ({ icon, title, subtitle }: OnboardingCardHeaderProps) => {
  return (
    <CardHeader className="border-b border-border/70 pb-5">
      <div className="rounded-2xl border border-primary/20 bg-linear-to-br from-primary/12 via-primary/4 to-transparent p-4 shadow-xs">
        <div className="flex items-start gap-4">
          <div className="flex size-14 shrink-0 items-center justify-center rounded-2xl bg-primary text-primary-foreground shadow-sm ring-1 ring-primary/30">
            {icon}
          </div>
          <div className="min-w-0 space-y-1">
            <CardTitle className="text-xl">{title}</CardTitle>
            <CardDescription className="max-w-2xl leading-6">{subtitle}</CardDescription>
          </div>
        </div>
      </div>
    </CardHeader>
  )
}
