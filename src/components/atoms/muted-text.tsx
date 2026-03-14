import { cn } from '@/lib/utils/cn'

export const MutedText = ({ children, className }: { children: React.ReactNode; className?: string }) => {
  return <span className={cn('text-muted-foreground/70', className)}>{children}</span>
}

export const MutedTextItalic = ({ children, className }: { children: React.ReactNode; className?: string }) => {
  return <span className={cn('text-muted-foreground/70 italic', className)}>{children}</span>
}

export const MutedTextSmall = ({ children, className }: { children: React.ReactNode; className?: string }) => {
  return <span className={cn('text-muted-foreground/70 text-sm', className)}>{children}</span>
}

export const MutedTextSmallItalic = ({ children, className }: { children: React.ReactNode; className?: string }) => {
  return <span className={cn('text-muted-foreground/70 text-sm italic', className)}>{children}</span>
}

export const MutedTextAccent = ({ children, className }: { children: React.ReactNode; className?: string }) => {
  return <span className={cn('text-muted-foreground', className)}>{children}</span>
}

export const MutedTextAccentItalic = ({ children, className }: { children: React.ReactNode; className?: string }) => {
  return <span className={cn('text-muted-foreground italic', className)}>{children}</span>
}

export const MutedTextAccentSmall = ({ children, className }: { children: React.ReactNode; className?: string }) => {
  return <span className={cn('text-muted-foreground text-sm', className)}>{children}</span>
}

export const MutedTextAccentSmallItalic = ({
  children,
  className,
}: {
  children: React.ReactNode
  className?: string
}) => {
  return <span className={cn('text-muted-foreground text-sm italic', className)}>{children}</span>
}
