import { Suspense, use } from 'react'
import { DockviewProvider } from '@/context/dockview-context'
import { SimpleLoader } from './components/atoms/simple-loader'
import { useAppViewModel } from './hooks/use-app-viewmodel'

interface AppProps {
  backendReady: Promise<void>
}

export const App = ({ backendReady }: AppProps) => {
  const appViewModel = useAppViewModel()

  if (appViewModel.isLoading) return <SimpleLoader />

  return (
    <Suspense fallback={<SimpleLoader withMessage={false} />}>
      <BackendReady backendReady={backendReady} />
    </Suspense>
  )
}

const BackendReady = ({ backendReady }: AppProps) => {
  use(backendReady)

  return <DockviewProvider />
}
