import { AnimatePresence, motion } from 'framer-motion'
import { observable, runInAction } from 'mobx'
import { useEffect } from 'react'
import { BgGrid } from './bg-grid'
import { BlprntLogo } from './blprnt-logo'
import { ShimmeringText } from './shimmering-text'

const loadingMessages = [
  'Consulting the vast library',
  'Weighing all possible answers',
  'Summoning relevant knowledge',
  'Sifting through everything ever written',
  'Rehearsing the response silently',
  'Choosing words carefully',
  'Contemplating the space between meanings',
  'Gathering thoughts from the archive',
  'Polishing the reasoning',
  'Listening harder than necessary',
  'Constructing a helpful disposition',
  'Dusting off obscure facts',
  'Organizing a coherent thought',
  'Simulating patience',
  'Reaching for the right analogy',
  'Untangling the question beneath the question',
  'Preparing a measured response',
  'Consulting the collective wisdom',
  'Filtering out the nonsense',
  'Composing with intent',
  'Mapping the shape of your request',
  'Locating the thread of relevance',
  'Imagining what you meant',
  'Steadying the thinking',
  'Assembling a point of view',
  'Rummaging through the archives',
  'Warming up the intuition',
  'Bridging concept to language',
  'Deciding where to begin',
  'Finding the thread',
  'Balancing brevity with thoroughness',
  'Resisting the urge to ramble',
  'Picking the scenic route through an explanation',
  'Suppressing irrelevant tangents',
]

const chosenMessages: string[] = []

// Choose a random message that has not been chosen yet
const chooseRandomMessage = () => {
  if (chosenMessages.length === loadingMessages.length) chosenMessages.length = 0

  const remainingMessages = loadingMessages.filter((message) => !chosenMessages.includes(message))
  const message = remainingMessages[Math.floor(Math.random() * remainingMessages.length)]

  chosenMessages.push(message)
  return message
}

const loadingMessage = observable.box(chooseRandomMessage())

let interval: number | null = null

const startInterval = () => {
  if (!interval) interval = setInterval(() => runInAction(() => loadingMessage.set(chooseRandomMessage())), 5000)
}

const stopInterval = () => {
  if (interval) clearInterval(interval)
  interval = null
}

export const SimpleLoader = ({ withMessage = true }: { withMessage?: boolean }) => {
  const message = withMessage ? loadingMessage.get() : ''

  useEffect(() => {
    startInterval()

    return () => stopInterval()
  }, [])

  return (
    <>
      <BgGrid />
      <div className="flex flex-col h-screen w-screen items-center justify-center absolute top-0 left-0 pointer-events-none translate-y-11.5">
        <BlprntLogoPing />

        <AnimatePresence mode="wait">
          <motion.div
            key={message ?? 'loading'}
            animate={{ opacity: 1, y: 0 }}
            className="text-2xl"
            exit={{ opacity: 0, y: -10 }}
            initial={{ opacity: 0, y: 10 }}
          >
            <ShimmeringText color="var(--primary)" text={message} />
          </motion.div>
        </AnimatePresence>
      </div>
    </>
  )
}

export const BlprntLogoPing = () => {
  return (
    <div className="size-12 transition-all animate-[ping_1500ms_cubic-bezier(0,0,0.2,1)_infinite] mb-15">
      <BlprntLogo />
    </div>
  )
}
