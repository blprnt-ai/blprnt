import { useState } from 'react'
import { AdvancedPageViewModel, AdvancedPageViewmodelContext } from './adcanced-page-viewmodel'
import { AdvancedPage } from './advanced-page'

export const AdvancedPageProvider = () => {
  const [viewmodel] = useState(() => new AdvancedPageViewModel())

  return (
    <AdvancedPageViewmodelContext.Provider value={viewmodel}>
      <AdvancedPage />
    </AdvancedPageViewmodelContext.Provider>
  )
}
