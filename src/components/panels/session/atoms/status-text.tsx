import { TextSweep } from '@/components/atoms/text-sweep'

interface StatusTextProps {
  text: string
}

export const StatusText = ({ text }: StatusTextProps) => {
  return (
    <div className="shrink-0 text-sm text-muted-foreground h-full flex">
      <div className="h-full flex items-center px-4">
        <TextSweep className="font-medium whitespace-nowrap overflow-hidden text-ellipsis italic">{text}</TextSweep>
      </div>
    </div>
  )
}
