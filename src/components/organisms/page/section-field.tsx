import type { PropsWithChildren } from 'react'

interface SectionFieldProps {
  title: React.ReactNode
}
export const SectionField = ({ title, children }: PropsWithChildren<SectionFieldProps>) => {
  return (
    <div className="flex gap-8 py-4 w-full items-start">
      <div className="flex items-start w-48 shrink-0">
        <div>{title}</div>
      </div>
      <div className="flex flex-col gap-4 items-start w-full">{children}</div>
    </div>
  )
}
