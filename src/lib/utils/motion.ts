import type { MotionProps, Transition } from 'motion/react'

export const defaultTransition: Transition = { duration: 0.3, ease: 'easeInOut' }

export const verticalMotionProps: MotionProps = {
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: -10 },
  initial: { opacity: 0, y: 10 },
  transition: defaultTransition,
}

export const horizontalMotionProps: MotionProps = {
  animate: { opacity: 1, x: 0 },
  exit: { opacity: 0, x: 10 },
  initial: { opacity: 0, x: -10 },
  transition: defaultTransition,
}

export const contextMenuMotionProps: MotionProps = {
  animate: { opacity: 1 },
  initial: { opacity: 0 },
  transition: defaultTransition,
}
