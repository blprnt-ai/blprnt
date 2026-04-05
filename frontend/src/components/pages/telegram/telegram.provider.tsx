import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { AppLoader } from '@/components/organisms/app-loader'
import { AppModel } from '@/models/app.model'
import { TelegramPage } from './telegram.page'
import { TelegramViewmodel, TelegramViewmodelContext } from './telegram.viewmodel'

export const TelegramProvider = observer(() => {
  const ownerId = AppModel.instance.owner?.id ?? null
  const [viewmodel, setViewmodel] = useState<TelegramViewmodel | null>(null)

  useEffect(() => {
    if (!ownerId) return

    const nextViewmodel = new TelegramViewmodel(ownerId)
    setViewmodel(nextViewmodel)
    void nextViewmodel.init()
  }, [ownerId])

  if (!viewmodel || viewmodel.isLoading) return <AppLoader />

  return (
    <TelegramViewmodelContext.Provider value={viewmodel}>
      <TelegramPage />
    </TelegramViewmodelContext.Provider>
  )
})