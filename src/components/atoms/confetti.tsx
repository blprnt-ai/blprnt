import type { GlobalOptions as ConfettiGlobalOptions, CreateTypes as ConfettiInstance } from 'canvas-confetti'
import confetti from 'canvas-confetti'
import type React from 'react'
import { useCallback, useEffect, useRef } from 'react'
import { asyncWait } from '@/lib/utils/misc'

type Props = React.ComponentPropsWithRef<'canvas'> & {
  globalOptions?: ConfettiGlobalOptions
}

export const Confetti = (props: Props) => {
  const { globalOptions = { resize: true, useWorker: true } } = props
  const instanceRef = useRef<ConfettiInstance | null>(null)

  const canvasRef = useCallback(
    (node: HTMLCanvasElement) => {
      if (node !== null) {
        if (instanceRef.current) return
        instanceRef.current = confetti.create(node, {
          ...globalOptions,
          resize: true,
        })
      } else {
        if (instanceRef.current) {
          instanceRef.current.reset()
          instanceRef.current = null
        }
      }
    },
    [globalOptions],
  )

  // biome-ignore lint/correctness/useExhaustiveDependencies: we need to access the instanceRef.current in the callback
  const fire = useCallback(
    async (opts = {}) => {
      if (!instanceRef.current) return

      try {
        await asyncWait(400)
        await instanceRef.current(opts)
      } catch (error) {
        console.error('Failed to fire confetti', error)
      }
    },
    [instanceRef.current],
  )

  // biome-ignore lint/correctness/useExhaustiveDependencies: we need to access the instanceRef.current in the callback
  useEffect(() => {
    if (!instanceRef.current) return

    fire()
  }, [fire, instanceRef.current])

  return (
    <canvas
      ref={canvasRef}
      className="absolute inset-0 w-full h-full z-70 pointer-events-none"
      height={window.innerHeight}
      width={window.innerWidth}
    />
  )
}
