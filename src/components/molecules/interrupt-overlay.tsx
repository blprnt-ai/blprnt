import { AlertTriangleIcon } from 'lucide-react'

export const InterruptOverlay = () => {
  return (
    <div className="absolute fade-in-anim-quick h-full w-full top-0 left-0 bg-destructive/15 text-destructive z-50 flex items-center justify-center pointer-none pointer-events-none">
      <div className="h-40 flex items-center justify-center gap-4 blink">
        <AlertTriangleIcon className="size-10" />
        <span className="text-3xl text-shadow-lg text-shadow-black">Press esc again to interrupt</span>
      </div>
    </div>
  )
}
