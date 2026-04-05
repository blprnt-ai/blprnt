import { Loader2 } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { useState } from 'react'
import type { ProviderDto } from '@/bindings/ProviderDto'
import { Button } from '@/components/ui/button'
import type { ProviderModel } from '@/models/provider.model'
import { ProviderFormViewmodel } from './provider.viewmodel'
import { ProviderFields } from './provider-fields'

interface ProviderFormProps {
  leftButtons?: React.ReactNode
  rightButtonText?: React.ReactNode
  provider?: ProviderDto | ProviderModel
  onProviderSaved: (provider: ProviderDto) => void
}

export const ProviderForm = observer(
  ({ leftButtons, rightButtonText, provider, onProviderSaved }: ProviderFormProps) => {
    const [viewmodel] = useState(() => new ProviderFormViewmodel(provider))
    const handleSave = async (e: React.FormEvent<HTMLFormElement>) => {
      e.preventDefault()

      const provider = await viewmodel.save()
      if (!provider) return

      onProviderSaved(provider)
    }

    const verb = viewmodel.provider.id ? 'Update' : 'Create'

    return (
      <form onSubmit={handleSave}>
        <div className="flex flex-col gap-6">
          <ProviderFields provider={viewmodel.provider} />

          <div className="flex justify-between">
            <div>{leftButtons}</div>

            <Button className="transition-all duration-300" disabled={!viewmodel.canSave} type="submit">
              {viewmodel.isSaving ? <Loader2 className="w-4 h-4 animate-spin" /> : rightButtonText || verb}
            </Button>
          </div>
        </div>
      </form>
    )
  },
)
