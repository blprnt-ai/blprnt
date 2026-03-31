import { Loader2 } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { useState } from 'react'
import type { Provider } from '@/bindings/Provider'
import type { ProviderDto } from '@/bindings/ProviderDto'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { cn } from '@/lib/utils'
import type { ProviderModel } from '@/models/provider.model'
import { ProviderFormViewmodel } from './provider.viewmodel'

interface ProviderOption {
  description: string
  provider: Provider
  title: string
}

const providers: ProviderOption[] = [
  {
    description: 'Use your claude-code subscription.',
    provider: 'claude_code',
    title: 'Claude Code',
  },
  {
    description: 'Use your codex subscription.',
    provider: 'codex',
    title: 'Codex',
  },
  {
    description: 'Use your Open Router API key.',
    provider: 'open_router',
    title: 'Open Router',
  },
  {
    description: 'Use your Anthropic API key.',
    provider: 'anthropic',
    title: 'Anthropic',
  },
  {
    description: 'Use your OpenAI API key.',
    provider: 'openai',
    title: 'Open AI',
  },
]

interface ProviderFormProps {
  leftButtons?: React.ReactNode
  rightButtonText?: React.ReactNode
  provider?: ProviderDto | ProviderModel
  onProviderSaved: (provider: ProviderDto) => void
}

export const ProviderForm = observer(
  ({ leftButtons, rightButtonText, provider, onProviderSaved }: ProviderFormProps) => {
    const [viewmodel] = useState(() => new ProviderFormViewmodel(provider))

    const handleSelectProvider = (provider: Provider) => (viewmodel.provider.provider = provider)
    const handleChangeApiKey = (apiKey: string) => (viewmodel.provider.apiKey = apiKey)
    const handleChangeBaseUrl = (baseUrl: string) => (viewmodel.provider.baseUrl = baseUrl)
    const handleSave = async (e: React.FormEvent<HTMLFormElement>) => {
      e.preventDefault()

      const provider = await viewmodel.save()
      if (!provider) return

      onProviderSaved(provider)
    }

    const verb = viewmodel.provider.id ? 'Update' : 'Create'

    return (
      <form onSubmit={handleSave}>
        <div className="flex flex-col gap-4">
          <div className="grid grid-cols-2 gap-4">
            {providers.map((provider) => (
              <ProviderCard
                key={provider.title}
                {...provider}
                selectedProvider={viewmodel.provider.provider}
                onSelect={handleSelectProvider}
              />
            ))}
          </div>

          {!viewmodel.provider.isOauthProvider && (
            <div className="flex flex-col gap-2">
              <div className="flex flex-col gap-2">
                <Label>API Key</Label>
                <Input
                  type="password"
                  value={viewmodel.provider.apiKey}
                  onChange={(e) => handleChangeApiKey(e.target.value)}
                />
              </div>
              <div className="flex flex-col gap-2">
                <Tooltip>
                  <Label>
                    <TooltipTrigger>
                      Base URL <span className="text-muted-foreground text-xs">(optional)</span>
                    </TooltipTrigger>
                  </Label>
                  <TooltipContent>
                    <div>This is only required if you're using a non-standard API endpoint for your provider.</div>
                  </TooltipContent>
                </Tooltip>
                <Input value={viewmodel.provider.baseUrl} onChange={(e) => handleChangeBaseUrl(e.target.value)} />
              </div>
            </div>
          )}

          {viewmodel.provider.isOauthProvider && (
            <div className="flex flex-col gap-2 text-sm text-muted-foreground font-light">
              You must have the {viewmodel.provider.provider === 'claude_code' ? 'claude-code' : 'codex'} cli installed
              and configured.
            </div>
          )}

          <div className="flex justify-between">
            <div>{leftButtons}</div>

            <Button
              className="transition-all duration-300"
              disabled={!viewmodel.provider.isValid || viewmodel.isSaving}
              type="submit"
            >
              {viewmodel.isSaving ? <Loader2 className="w-4 h-4 animate-spin" /> : rightButtonText || verb}
            </Button>
          </div>
        </div>
      </form>
    )
  },
)

interface ProviderCardProps extends ProviderOption {
  selectedProvider: Provider
  onSelect: (provider: Provider) => void
}

const ProviderCard = ({ title, description, provider, selectedProvider, onSelect }: ProviderCardProps) => {
  return (
    <div
      className={cn(
        'flex flex-col gap-2 border rounded-md p-4 cursor-pointer',
        selectedProvider === provider
          ? 'border-primary'
          : 'brightness-75 hover:brightness-100 hover:border-white/70  duration-300',
      )}
      onClick={() => onSelect(provider)}
    >
      <div className="flex flex-col gap-2">
        <Label>{title}</Label>
      </div>
      <div className="flex flex-col gap-2 text-sm text-muted-foreground font-light">{description}</div>
    </div>
  )
}
