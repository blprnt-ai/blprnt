import { motion } from 'framer-motion'
import { cn } from '@/lib/utils/cn'

interface SessionChipProps {
  label?: string
  value?: string | null
  className?: string
}

export const SessionChip = ({ label, value, className }: SessionChipProps) => {
  if (!value) return null

  return (
    <div
      className={cn(
        'flex gap-1 items-center justify-center border-r h-full px-4 text-sm text-muted-foreground',
        className,
      )}
    >
      {label && <span className="text-muted-foreground/70 whitespace-nowrap">{label}: </span>}
      <motion.span key={value} animate={{ opacity: 1 }} className="whitespace-nowrap" initial={{ opacity: 0 }}>
        {value}
      </motion.span>
    </div>
  )
}
