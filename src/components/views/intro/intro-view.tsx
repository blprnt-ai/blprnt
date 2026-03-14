import { AnimatePresence, motion } from 'motion/react'
// @ts-expect-error
import { Typewriter } from 'motion-plus/react'
import { useEffect, useState } from 'react'
import { Button } from '@/components/atoms/button'
import { useAppViewModel } from '@/hooks/use-app-viewmodel'
import { useAudio } from '@/hooks/use-audio'
import { useBlprntConfig } from '@/lib/utils/blprnt-config'

let hasPlayed = false

const lines = ['Dream it.', 'Draft it.', 'Do it.', 'Done.']

export const IntroView = () => {
  const appStore = useAppViewModel()
  const config = useBlprntConfig()
  const [isDone, setIsDone] = useState(true)

  const blprnt = useAudio('/sounds/blprnt.wav')

  // biome-ignore lint/correctness/useExhaustiveDependencies: don't depend on config
  useEffect(() => {
    config.setSeenIntroScreen(true)
  }, [])

  useEffect(() => {
    if (hasPlayed) return
    hasPlayed = true

    setTimeout(() => blprnt.play(), 500)
  }, [blprnt])

  const [currentLine, setCurrentLine] = useState(-1)

  // biome-ignore lint/correctness/useExhaustiveDependencies: don't depend on isDone
  useEffect(() => {
    if (currentLine === lines.length - 1) {
      setTimeout(() => setIsDone(true), 5000)

      return
    }

    if (isDone) setTimeout(() => setIsDone(false), 1500)

    setTimeout(() => setCurrentLine(currentLine + 1), 2500)
  }, [currentLine])

  return (
    <div className="flex flex-col justify-center h-screen w-screen bg-grid-2">
      <div className="flex flex-col h-full items-center justify-center gap-4 pointer-events-none select-none">
        <div className="text-5xl font-light flex items-center gap-2">
          Meet <span className="text-primary font-medium font-mono">blprnt</span>!
        </div>

        <AnimatePresence mode="wait">
          <motion.div
            key={isDone ? 'done' : 'typing'}
            layout
            animate={{ maxHeight: isDone ? 0 : '8rem', opacity: 1, x: 0 }}
            className="text-2xl font-light"
            exit={{ maxHeight: isDone ? '8rem' : 0, opacity: 0, x: 20 }}
            initial={{ maxHeight: isDone ? 0 : '8rem', opacity: 0, x: -20 }}
          >
            {!isDone && <Typewriter>{lines[currentLine]}</Typewriter>}
          </motion.div>
        </AnimatePresence>

        <Button className="italic pointer-events-auto" size="xl" variant="outline" onClick={() => appStore.setReady()}>
          Let's Go!
        </Button>
      </div>
    </div>
  )
}
