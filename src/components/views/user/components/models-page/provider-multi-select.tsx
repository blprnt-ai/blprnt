import { Check, ChevronDown } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { cn } from '@/lib/utils/cn'

export const ProviderMultiSelect = ({
  providers,
  selected,
  onToggle,
}: {
  providers: string[]
  selected: string[]
  onToggle: (provider: string) => void
}) => {
  const [isOpen, setIsOpen] = useState(false)
  const containerRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(event.target as Node)) {
        setIsOpen(false)
      }
    }

    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [])

  const displayText =
    selected.length === 0 ? 'All' : selected.length === 1 ? selected[0] : `${selected.length} selected`

  return (
    <div ref={containerRef} className="relative">
      <button
        type="button"
        className={cn(
          'flex items-center justify-between gap-2 w-36 h-9',
          'rounded-md border border-input bg-transparent px-3 py-2 text-sm',
          'transition-[color,box-shadow] outline-none cursor-pointer',
          'hover:bg-muted/50',
        )}
        onClick={() => setIsOpen(!isOpen)}
      >
        <span className="truncate">{displayText}</span>
        <ChevronDown className={cn('size-4 opacity-50 shrink-0 transition-transform', isOpen && 'rotate-180')} />
      </button>

      {isOpen && (
        <div className="absolute top-full left-0 z-50 mt-1 w-48 max-h-64 overflow-y-auto rounded-md border bg-popover p-1 shadow-md">
          {providers.map((provider) => {
            const isSelected = selected.includes(provider)
            return (
              <button
                key={provider}
                type="button"
                className={cn(
                  'flex w-full items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-none',
                  'hover:bg-primary hover:text-primary-foreground',
                  'cursor-pointer select-none',
                )}
                onClick={() => onToggle(provider)}
              >
                <div
                  className={cn(
                    'flex size-4 shrink-0 items-center justify-center rounded-[4px] border',
                    isSelected && 'bg-primary border-primary text-primary-foreground',
                  )}
                >
                  {isSelected && <Check className="size-3" />}
                </div>
                <span className="capitalize">{provider}</span>
              </button>
            )
          })}
        </div>
      )}
    </div>
  )
}
