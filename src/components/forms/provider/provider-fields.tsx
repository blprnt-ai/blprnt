import { observer } from 'mobx-react-lite'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { cn } from '@/lib/utils'
import type { ProviderModel } from '@/models/provider.model'
import { isOauthProvider, PROVIDER_OPTIONS } from './provider-catalog'

interface ProviderFieldsProps {
  provider: ProviderModel
  showProviderSelector?: boolean
}

export const ProviderFields = observer(({ provider, showProviderSelector = true }: ProviderFieldsProps) => {
  return (
    <div className="flex flex-col gap-4">
      {showProviderSelector && (
        <div className="grid gap-4 sm:grid-cols-2">
          {PROVIDER_OPTIONS.map((option) => (
            <button
              key={option.provider}
              className={cn(
                'flex cursor-pointer flex-col gap-2 rounded-md border p-4 text-left transition-colors',
                provider.provider === option.provider
                  ? 'border-primary bg-primary/5'
                  : 'border-border/70 hover:border-primary/25 hover:bg-muted/40',
              )}
              type="button"
              onClick={() => {
                provider.provider = option.provider
              }}
            >
              <Label>{option.title}</Label>
              <div className="text-sm font-light text-muted-foreground">{option.description}</div>
            </button>
          ))}
        </div>
      )}

      {!isOauthProvider(provider.provider) ? (
        <div className="flex flex-col gap-2">
          <div className="flex flex-col gap-2">
            <Label>API Key</Label>
            <Input
              type="password"
              value={provider.apiKey}
              onChange={(event) => (provider.apiKey = event.target.value)}
            />
          </div>

          <div className="flex flex-col gap-2">
            <Label>
              <Tooltip>
                <TooltipTrigger>
                  Base URL <span className="text-xs text-muted-foreground">(optional)</span>
                </TooltipTrigger>
                <TooltipContent>
                  <div>This is only required if you're using a non-standard API endpoint for your provider.</div>
                </TooltipContent>
              </Tooltip>
            </Label>
            <Input value={provider.baseUrl} onChange={(event) => (provider.baseUrl = event.target.value)} />
          </div>
        </div>
      ) : (
        <div className="text-sm font-light text-muted-foreground">
          You must have the {provider.provider === 'claude_code' ? 'claude-code' : 'codex'} cli installed and
          configured.
        </div>
      )}
    </div>
  )
})
