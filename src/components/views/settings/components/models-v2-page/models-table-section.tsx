import type { ReactNode } from 'react'

interface ModelsTableSectionProps {
  title: string
  description: string
  children: ReactNode
}

export const ModelsTableSection = ({ title, description, children }: ModelsTableSectionProps) => {
  return (
    <div className="w-full space-y-2">
      <div className="space-y-1">
        <div className="text-sm font-semibold">{title}</div>
        <div className="text-muted-foreground text-xs font-light">{description}</div>
      </div>
      {children}
    </div>
  )
}