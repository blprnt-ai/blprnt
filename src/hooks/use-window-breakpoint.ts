import { useCallback, useEffect, useState } from 'react'

export enum Breakpoint {
  xs = 'xs',
  sm = 'sm',
  md = 'md',
  lg = 'lg',
  xl = 'xl',
  '2xl' = '2xl',
  '3xl' = '3xl',
  '4xl' = '4xl',
}

const BREAKPOINTS = {
  [Breakpoint.sm]: 1024,
  [Breakpoint.md]: 1280,
  [Breakpoint.lg]: 1536,
  [Breakpoint.xl]: 1920,
  [Breakpoint['2xl']]: 2560,
  [Breakpoint['3xl']]: 3840,
  [Breakpoint['4xl']]: 5120,
}

const getBreakpointName = (width: number) => {
  if (width >= BREAKPOINTS[Breakpoint['4xl']]) return Breakpoint['4xl']
  if (width >= BREAKPOINTS[Breakpoint['3xl']]) return Breakpoint['3xl']
  if (width >= BREAKPOINTS[Breakpoint['2xl']]) return Breakpoint['2xl']
  if (width >= BREAKPOINTS[Breakpoint.xl]) return Breakpoint.xl
  if (width >= BREAKPOINTS[Breakpoint.lg]) return Breakpoint.lg
  if (width >= BREAKPOINTS[Breakpoint.md]) return Breakpoint.md
  if (width >= BREAKPOINTS[Breakpoint.sm]) return Breakpoint.sm
  return Breakpoint.xs
}

export function useBreakpoint() {
  const [breakpoint, setBreakpoint] = useState(() => {
    if (typeof window !== 'undefined') return getBreakpointName(window.innerWidth)

    return Breakpoint.xs
  })

  const handleResize = useCallback(() => {
    const newBreakpoint = getBreakpointName(window.innerWidth)

    if (newBreakpoint !== breakpoint) setBreakpoint(newBreakpoint)
  }, [breakpoint])

  useEffect(() => {
    handleResize()

    window.addEventListener('resize', handleResize)

    return () => {
      window.removeEventListener('resize', handleResize)
    }
  }, [handleResize])

  return breakpoint
}
