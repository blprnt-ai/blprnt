import type { Provider } from '@/bindings'
import { ProviderIcon } from '@/components/atoms/provider-icon'
import { cn } from '@/lib/utils/cn'
import type { AllProviders } from '@/types'

interface ProviderCardProps {
  selected?: boolean
  provider: Provider
  onClick?: () => void
  className?: string
}

const getEnabledProviderDetails = (provider: AllProviders) => {
  switch (provider) {
    case 'anthropic':
    case 'anthropic_fnf':
      return {
        icon: <ProviderIcon provider="anthropic" />,
        title: provider === 'anthropic_fnf' ? 'Claude-Code' : 'Anthropic',
        variant: provider === 'anthropic_fnf' ? 'dark' : 'normal',
      }
    case 'openai':
    case 'openai_fnf':
      return {
        icon: <ProviderIcon provider="openai" />,
        title: provider === 'openai_fnf' ? 'Codex' : 'OpenAI',
        variant: provider === 'openai_fnf' ? 'dark' : 'normal',
      }
    case 'open_router':
      return {
        title: <span className="text-3xl font-medium font-mono text-primary">blprnt</span>,
        variant: 'normal',
      }
    default:
      return null
  }
}

export const ProviderCard = ({ provider, selected = false, onClick, className }: ProviderCardProps) => {
  const details = getEnabledProviderDetails(provider)
  if (!details) return null

  const { icon, title, variant } = details

  return (
    <button
      type="button"
      className={cn(
        'flex flex-col gap-4 rounded-xl border-2 bg-card p-6 font text-left transition-all duration-300 hover:shadow-2xl ring-1 ring-transparent cursor-pointer w-72 hover:translate-y-[-2px]',
        selected ? 'border-primary ring-primary/60 shadow-2xl' : 'border-border',
        variant === 'dark' && 'bg-stone-950 border-red-900 ring-red-950 ',
        selected && variant === 'dark' && 'ring-red-500',
        provider === 'open_router' && 'w-152',
        className,
      )}
      onClick={onClick}
    >
      <div className={cn('flex items-center gap-3', provider === 'open_router' && 'justify-center w-full')}>
        {icon && <div className="flex h-12 w-12 items-center justify-center rounded-lg">{icon}</div>}
        <div>
          <div className="text-lg font-medium text-foreground">{title}</div>
        </div>
      </div>
    </button>
  )
}
