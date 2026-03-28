import { Loader2Icon } from 'lucide-react'
import { motion } from 'motion/react'

export const AppLoader = () => {
  return (
    <motion.div
      animate={{ opacity: 1 }}
      className="flex h-full w-full max-h-screen max-w-screen items-center justify-center gap-2"
      exit={{ opacity: 0 }}
      initial={{ opacity: 0 }}
    >
      <div>
        <Loader2Icon className="size-4 animate-spin text-cyan-400" />
      </div>
      <div>Loading...</div>
    </motion.div>
  )
}
