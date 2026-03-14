import { getName, getVersion } from '@tauri-apps/api/app'
import { useEffect, useState } from 'react'
import { useAppViewModel } from '@/hooks/use-app-viewmodel'
import { useIsDev } from '@/hooks/use-is-dev'
import { cn } from '@/lib/utils/cn'

const version = await getVersion()
const name = await getName()

export const BlprntVersion = () => {
  const appStore = useAppViewModel()
  const isDev = useIsDev()
  const [build, setBuild] = useState('')

  useEffect(() => {
    appStore.appModel
      .buildHash()
      .then((buildHash) => {
        setBuild(buildHash)
      })
      .catch((error) => {
        console.error('Error getting build hash', error)
      })
  }, [appStore.appModel])

  return (
    <div className={cn('text-sm text-muted-foreground flex items-center gap-2 w-full')}>
      <span className="text-primary-dimmed whitespace-nowrap">{name}</span>
      <span className="flex justify-between w-full">
        <span className="whitespace-nowrap">
          v{version}
          {build && ` (${build})`}
        </span>
        {isDev && <span className="text-warn font-medium whitespace-nowrap">(dev-build)</span>}
      </span>
    </div>
  )
}
