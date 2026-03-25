import { Loader2Icon } from 'lucide-react'

export const AppLoader = () => {
  return (
    <div className="flex h-screen w-screen items-center justify-center gap-2">
      <div>
        <Loader2Icon className="size-4 animate-spin text-cyan-400" />
      </div>
      <div>Loading...</div>
    </div>
  )
}
