import { observer } from 'mobx-react-lite'
import { ProviderSheet } from '@/components/forms/provider/provider-sheet'
import { Page } from '@/components/layouts/page'
import { Card, CardContent } from '@/components/ui/card'
import { ProviderLibraryCard } from './components/provider-library-card'
import { useProvidersViewmodel } from './providers.viewmodel'

export const ProvidersPage = observer(() => {
  const viewmodel = useProvidersViewmodel()

  return (
    <Page className="overflow-y-auto px-3 pb-6 md:px-5">
      <div className="mx-auto flex w-full max-w-7xl flex-col gap-4">
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
    </Page>
  )
})
