import { motion } from 'framer-motion'
import { cn } from '@/lib/utils/cn'

interface GlassCardProps {
  className?: string
  children: React.ReactNode
  role?: string
  onClick?: () => void
}

export const GlassCard = ({ className, role, ...props }: GlassCardProps) => {
  return (
    <motion.div
      animate={{ opacity: 1, y: 0 }}
      initial={{ opacity: 0, y: 10 }}
      role={role}
      transition={{ duration: 0.3 }}
      className={cn(
        'flex flex-col gap-2 border border-border rounded-md p-4 hover:bg-white/5 hover:border-primary/80 transition-colors backdrop-blur-sm bg-accent',
        className,
      )}
      {...props}
    />
  )
}
