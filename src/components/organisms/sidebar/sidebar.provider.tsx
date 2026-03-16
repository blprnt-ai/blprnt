import { type PropsWithChildren, useEffect, useState } from 'react'
import { BlprntLogoPing } from '@/components/atoms/simple-loader'
import { useDockviewLayoutViewModel } from '@/components/dockview/dockview-layout.viewmodel'
import { useAppViewModel } from '@/hooks/use-app-viewmodel'
import { cn } from '@/lib/utils/cn'
import { SidebarViewmodel, SidebarViewmodelContext } from './sidebar.viewmodel'

export const SidebarProvider = ({ children }: PropsWithChildren) => {
  const appStore = useAppViewModel()
  const dockviewLayout = useDockviewLayoutViewModel()
  const [viewModel, setViewModel] = useState<SidebarViewmodel | null>(null)

  // biome-ignore lint/correctness/useExhaustiveDependencies: Only run on first render
  useEffect(() => {
    const viewModel = new SidebarViewmodel(appStore, dockviewLayout)
    setViewModel(viewModel)
    viewModel.init()

    return () => viewModel.destroy()
  }, [])

  if (!viewModel) return <LoadingState />

  return <SidebarViewmodelContext.Provider value={viewModel}>{children}</SidebarViewmodelContext.Provider>
}

const LoadingState = () => {
  return (
    <div
      className={cn(
        'flex justify-center items-center h-full w-full bg-background rounded-lg border overflow-hidden pb-1 bg-gradient-glow-dark',
      )}
    >
      <BlprntLogoPing />
    </div>
  )
}
