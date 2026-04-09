import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { AppLoader } from '@/components/organisms/app-loader'
import { MyWorkPage } from './my-work.page'
import { MyWorkViewmodel } from './my-work.viewmodel'

export const MyWorkProvider = observer(() => {
  const [viewmodel, setViewmodel] = useState(() => new MyWorkViewmodel())

  useEffect(() => {
    const nextViewmodel = new MyWorkViewmodel()
    setViewmodel(nextViewmodel)
    void nextViewmodel.init()
  }, [])

  if (viewmodel.isLoading) return <AppLoader />

  return <MyWorkPage viewmodel={viewmodel} />
})
