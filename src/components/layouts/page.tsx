import { motion } from 'motion/react'

interface PageProps {
  children: React.ReactNode
  className?: string
}

export const Page = ({ children, className }: PageProps) => {
  return (
    <motion.div animate={{ opacity: 1 }} className={className} exit={{ opacity: 0 }} initial={{ opacity: 0 }}>
      {children}
    </motion.div>
  )
}
