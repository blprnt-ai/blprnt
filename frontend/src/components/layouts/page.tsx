import { motion } from 'motion/react'
import { forwardRef } from 'react'
import { cn } from '@/lib/utils'

interface PageProps {
  children: React.ReactNode
  className?: string
}

export const Page = forwardRef<HTMLDivElement, PageProps>(({ children, className }, ref) => {
  return (
    <motion.div
      ref={ref}
      animate={{ opacity: 1 }}
      className={cn('min-w-0 pt-[11px]! max-h-[calc(100vh-3.625rem)] overflow-x-hidden', className)}
      exit={{ opacity: 0 }}
      initial={{ opacity: 0 }}
    >
      {children}
    </motion.div>
  )
})

Page.displayName = 'Page'
