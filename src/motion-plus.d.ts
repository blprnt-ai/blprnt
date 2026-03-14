declare module 'motion-plus/react' {
  import type { ReactNode } from 'react'
  export function AnimateActivity(props: AnimateActivityProps): ReactNode
}

declare module 'motion-plus/animate-activity' {
  import type { PropsWithChildren } from 'react'

  interface AnimateActivityProps extends PropsWithChildren {
    mode?: 'visible' | 'hidden'
    layoutMode?: 'pop' | 'sync'
  }
  export function AnimateActivity(props: AnimateActivityProps): React.ReactNode
}
