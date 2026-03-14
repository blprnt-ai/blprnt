import type { PropsWithChildren } from 'react'

export const Note = ({ children }: PropsWithChildren) => {
  return <p className="text-xs text-muted-foreground">{children}</p>
}
