import { CardDescription, CardHeader } from '@/components/ui/card'

interface OnboardingCardHeaderProps {
  icon: React.ReactNode
  title: React.ReactNode
  subtitle: React.ReactNode
}

export const OnboardingCardHeader = ({ icon, title, subtitle }: OnboardingCardHeaderProps) => {
  return (
    <CardHeader>
      <CardDescription className="flex items-center gap-4 border border-primary/30 rounded-md p-3 bg-black/5 dark:bg-black/80">
        <div className="text-primary-foreground bg-primary/80 rounded-full p-2">{icon}</div>
        <div className="flex flex-col">
          <span className="text-foreground text-lg">{title}</span>
          <span className="font-light">{subtitle}</span>
        </div>
      </CardDescription>
    </CardHeader>
  )
}
