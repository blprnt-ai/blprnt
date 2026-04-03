import { useState } from 'react'
import type { ProviderDto } from '@/bindings/ProviderDto'
import { isOauthProvider, type ProviderOption } from '@/components/forms/provider/provider-catalog'
import { ConfirmationDialog } from '@/components/molecules/confirmation-dialog'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'

interface ProviderLibraryCardProps {
  isDeleting: boolean
  option: ProviderOption
  provider: ProviderDto | null
  onDelete: () => Promise<void> | void
  onOpen: () => void
}

export const ProviderLibraryCard = ({ isDeleting, option, provider, onDelete, onOpen }: ProviderLibraryCardProps) => {
  const [isConfirmationOpen, setIsConfirmationOpen] = useState(false)
  const isConnected = provider !== null

  return (
    <>
      <Card className="border-border/60 py-0">
        <CardContent className="flex h-full flex-col gap-5 px-5 py-5">
          <div className="flex items-start justify-between gap-3">
            <div className="min-w-0 space-y-2">
              <div className="font-medium">{option.title}</div>
              <p className="text-sm text-muted-foreground">{option.description}</p>
            </div>
            <span
              className={
                isConnected
                  ? 'rounded-full bg-primary/10 px-2 py-1 text-xs text-primary'
                  : 'rounded-full bg-muted px-2 py-1 text-xs text-muted-foreground'
              }
            >
              {isConnected ? 'Connected' : 'Available'}
            </span>
          </div>

          <div className="grid gap-3 text-sm">
            <div className="rounded-sm border border-border/60 bg-background/75 p-3">
              <p className="text-xs uppercase tracking-[0.18em] text-muted-foreground">Connection</p>
              <p className="mt-2 text-muted-foreground">{getConnectionLabel(option, isConnected)}</p>
            </div>

            <div className="rounded-sm border border-border/60 bg-background/75 p-3">
              <p className="text-xs uppercase tracking-[0.18em] text-muted-foreground">Endpoint</p>
              <p className="mt-2 break-all text-muted-foreground">{getEndpointLabel(option, provider)}</p>
            </div>
          </div>

          <div className="mt-auto flex items-center justify-between gap-3">
            <div>
              {isConnected ? (
                <Button
                  disabled={isDeleting}
                  size="sm"
                  type="button"
                  variant="destructive-outline"
                  onClick={() => setIsConfirmationOpen(true)}
                >
                  {isDeleting ? 'Removing...' : 'Remove'}
                </Button>
              ) : null}
            </div>

            <Button size="sm" type="button" variant={isConnected ? 'outline' : 'secondary'} onClick={onOpen}>
              {isConnected ? 'Manage' : 'Connect'}
            </Button>
          </div>
        </CardContent>
      </Card>

      <ConfirmationDialog
        cancelLabel="Keep provider"
        confirmLabel="Remove provider"
        description={`${option.title} will be removed from this workspace.`}
        open={isConfirmationOpen}
        title={`Remove ${option.title}?`}
        onConfirm={() => {
          setIsConfirmationOpen(false)
          void onDelete()
        }}
        onOpenChange={setIsConfirmationOpen}
      />
    </>
  )
}

const getConnectionLabel = (option: ProviderOption, isConnected: boolean) => {
  if (!isConnected) return 'Ready to connect.'
  if (isOauthProvider(option.provider)) return 'Connected through your local CLI session.'

  return 'Connected with a stored API key.'
}

const getEndpointLabel = (option: ProviderOption, provider: ProviderDto | null) => {
  if (provider?.base_url) return provider.base_url
  if (isOauthProvider(option.provider)) return 'Managed by the local CLI.'

  return 'Default provider endpoint.'
}
