import { Page } from '@/components/layouts/page'
import { ProvidersContent } from './providers.content'

export const ProvidersPage = () => {
  return (
    <Page className="overflow-y-auto px-3 pb-6 md:px-5">
      <div className="mx-auto w-full max-w-7xl">
        <ProvidersContent />
      </div>
    </Page>
  )
}
