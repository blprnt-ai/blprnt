import { motion } from 'motion/react'
import { cn } from '@/lib/utils'

interface PageProps {
  children: React.ReactNode
  className?: string
}

export const Page = ({ children, className }: PageProps) => {
  return (
    <motion.div
      animate={{ opacity: 1 }}
      className={cn('pt-[11px]! max-h-[calc(100vh-3.625rem)]', className)}
      exit={{ opacity: 0 }}
      initial={{ opacity: 0 }}
    >
      {children}
    </motion.div>
  )
}
