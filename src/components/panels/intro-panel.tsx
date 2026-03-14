import { AnimatePresence, motion } from 'framer-motion'
import { NoSession } from '@/components/organisms/no-session'

export const IntroPanel = () => {
  return (
    <AnimatePresence mode="popLayout">
      <motion.div
        animate={{ opacity: 1 }}
        className="flex flex-col items-center justify-center h-full"
        initial={{ opacity: 0 }}
      >
        <NoSession />
      </motion.div>
    </AnimatePresence>
  )
}
