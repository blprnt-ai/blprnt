import { ChevronDown } from 'lucide-react'
import { useEffect } from 'react'
import { Disclosure, DisclosureContent, DisclosureTrigger } from '@/components/atoms/disclosure'
import { useDisclosure } from '@/hooks/use-disclosure'
import { cn } from '@/lib/utils/cn'

interface ProjectDescriptionProps {
  className?: string
  description: string
  disclosure: React.ReactNode
  onClose?: () => void
}

export const TextWithDescription = ({ className, description, disclosure, onClose }: ProjectDescriptionProps) => {
  return (
    <Disclosure>
      <InnerDisclosure className={className} description={description} disclosure={disclosure} onClose={onClose} />
    </Disclosure>
  )
}

const InnerDisclosure = ({ className, description, disclosure, onClose = () => {} }: ProjectDescriptionProps) => {
  const { open } = useDisclosure()

  useEffect(() => {
    if (!open) onClose()
  }, [open, onClose])

  return (
    <>
      <div className={cn('flex gap-2 items-center mb-6')}>
        <div className="text-muted-foreground font-normal whitespace-nowrap">
          <p>{description}</p>
        </div>
        <DisclosureTrigger>
          <button className="w-full flex items-center gap-1 text-primary/60" type="button">
            read more
            <ChevronDown className={cn('mt-[2px] size-4 transition-transform', open && 'rotate-180')} />
          </button>
        </DisclosureTrigger>
      </div>
      <DisclosureContent>
        <div className={cn('overflow-hidden py-3 px-6 border rounded-md w-2xl mb-6', className)}>
          <div className="font-mono text-sm">{disclosure}</div>
        </div>
      </DisclosureContent>
    </>
  )
}
