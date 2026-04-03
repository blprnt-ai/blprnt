import { observer } from 'mobx-react-lite'
import { ProviderSheet } from '@/components/forms/provider/provider-sheet'
import { Card, CardContent } from '@/components/ui/card'
import { ProviderLibraryCard } from './components/provider-library-card'
import { useProvidersViewmodel } from './providers.viewmodel'

export const ProvidersContent = observer(() => {
  const viewmodel = useProvidersViewmodel()

  return (
    <div className="flex flex-col gap-4">
      {viewmodel.errorMessage ? (
        <Card>
          <CardContent className="py-4 text-sm text-destructive">{viewmodel.errorMessage}</CardContent>
        </Card>
      ) : null}

      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
        {viewmodel.catalogEntries.map(({ option, provider }) => (
          <ProviderLibraryCard
            key={option.provider}
            isDeleting={viewmodel.isDeletingProviderId === provider?.id}
            option={option}
            provider={provider}
            onDelete={() => (provider ? viewmodel.deleteProvider(provider.id) : Promise.resolve())}
            onOpen={() => viewmodel.openProvider(option)}
          />
        ))}
      </div>

      <ProviderSheet viewmodel={viewmodel.sheet} />
    </div>
  )
})