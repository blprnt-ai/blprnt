import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { AppLoader } from '@/components/organisms/app-loader'
import { TelegramContent } from '@/components/pages/telegram/telegram.content'
import { TelegramViewmodel, TelegramViewmodelContext } from '@/components/pages/telegram/telegram.viewmodel'
import { AppModel } from '@/models/app.model'

export const TelegramSettingsSection = observer(() => {
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
      <TelegramContent />
    </TelegramViewmodelContext.Provider>
  )
})
