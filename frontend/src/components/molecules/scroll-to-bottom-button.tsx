import { ArrowDownIcon } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

interface ScrollToBottomButtonProps {
  className?: string
  onClick: () => void
  visible: boolean
}

export const ScrollToBottomButton = ({ className, onClick, visible }: ScrollToBottomButtonProps) => {
  return (
    <div
      className={cn(
        'pointer-events-none fixed right-4 bottom-4 z-20 flex justify-end transition-opacity duration-200 md:right-6',
        visible ? 'opacity-100' : 'opacity-0',
        className,
      )}
    >
      <Button
        aria-label="Scroll to bottom"
        className="pointer-events-auto rounded-full shadow-lg"
        onClick={onClick}
        size="icon"
        type="button"
        variant="secondary"
      >
        <ArrowDownIcon className="size-4" />
      </Button>
    </div>
  )
}