import { type HTMLMotionProps, motion } from 'motion/react'

import { SlotPrimitive, type WithAsChild } from '@/components/ui/slot'

export type ButtonPrimitiveProps = WithAsChild<HTMLMotionProps<'button'>>

export const ButtonPrimitive = ({ asChild = false, ...props }: ButtonPrimitiveProps) => {
  const Component = asChild ? SlotPrimitive : motion.button

  return <Component {...props} />
}
