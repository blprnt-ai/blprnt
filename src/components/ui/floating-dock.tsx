'use client'

import { IconLayoutNavbarCollapse } from '@tabler/icons-react'
import { Link } from '@tanstack/react-router'
import { AnimatePresence, motion } from 'motion/react'
import { useState } from 'react'
import { cn } from '@/lib/utils'

interface FloatingDockItem {
  title: string
  icon: React.ReactNode
  href: string
  isActive?: boolean
}

interface FloatingDockProps {
  items: FloatingDockItem[]
  mobileClassName?: string
}

export const FloatingDock = ({ items, mobileClassName }: FloatingDockProps) => {
  const [open, setOpen] = useState(false)

  return (
    <div className={cn('relative block md:hidden', mobileClassName)}>
      <AnimatePresence>
        {open && (
          <motion.div className="absolute inset-x-0 bottom-full mb-2 flex flex-col gap-2" layoutId="mobile-nav">
            {items.map((item, idx) => (
              <motion.div
                key={item.title}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, transition: { delay: idx * 0.05 }, y: 10 }}
                initial={{ opacity: 0, y: 10 }}
                transition={{ delay: (items.length - 1 - idx) * 0.05 }}
              >
                <Link
                  to={item.href}
                  className={cn(
                    'flex h-10 w-10 items-center justify-center rounded-full bg-neutral-900 text-neutral-100 ring-1 ring-black/20 shadow-sm transition-colors dark:bg-neutral-800 dark:text-neutral-100 dark:ring-white/10',
                    item.isActive &&
                      'bg-primary text-primary-foreground ring-primary/30 shadow-[0_10px_24px_-14px_color-mix(in_oklab,var(--primary)_80%,black)] dark:bg-primary dark:text-primary-foreground',
                  )}
                  onClick={() => setOpen(false)}
                >
                  <div className="h-4 w-4">{item.icon}</div>
                </Link>
              </motion.div>
            ))}
          </motion.div>
        )}
      </AnimatePresence>

      <button
        className="flex h-10 w-10 items-center justify-center rounded-full bg-neutral-900 text-neutral-100 ring-1 ring-black/20 shadow-sm dark:bg-neutral-800 dark:text-neutral-100 dark:ring-white/10"
        type="button"
        onClick={() => setOpen((current) => !current)}
      >
        <IconLayoutNavbarCollapse className="h-5 w-5 text-neutral-300 dark:text-neutral-200" />
      </button>
    </div>
  )
}
