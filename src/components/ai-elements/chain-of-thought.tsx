import { Braces, ChevronDownIcon, DotIcon, type LucideIcon } from 'lucide-react'
import type { ComponentProps, ReactNode } from 'react'
import { memo } from 'react'
import { StickToBottom } from 'use-stick-to-bottom'
import { Badge } from '@/components/atoms/badge'
import { Disclosure, DisclosureContent, DisclosureTrigger } from '@/components/atoms/disclosure'
import { useDisclosure } from '@/hooks/use-disclosure'
import { cn } from '@/lib/utils/cn'

export type ChainOfThoughtProps = ComponentProps<'div'> & {
  defaultOpen?: boolean
  isRounded?: boolean
}

export const ChainOfThought = memo(
  ({ className, defaultOpen = false, isRounded = false, children, ...props }: ChainOfThoughtProps) => {
    return (
      <Disclosure
        open={defaultOpen}
        className={cn(
          'bg-accent hover:border-primary/50 transition-colors duration-300',
          isRounded && 'rounded-lg border border-border',
          !isRounded && 'border-t border-b border-t-transparent border-b-border p-1.5',
          className,
        )}
        {...props}
      >
        {children}
      </Disclosure>
    )
  },
)

export type ChainOfThoughtHeaderProps = Omit<ComponentProps<typeof DisclosureTrigger>, 'children'>

export const ChainOfThoughtHeader = memo(({ className, ...props }: ChainOfThoughtHeaderProps) => {
  const { open } = useDisclosure()

  return (
    <DisclosureTrigger>
      <div
        className={cn(
          'p-2 flex w-full items-center gap-2 text-muted-foreground text-sm transition-colors hover:text-foreground hover:[&>span]:text-foreground/80',
          className,
        )}
        {...props}
      >
        <Braces className="size-4 text-primary" />
        <span className="flex-1 text-right text-muted-foreground/50 transition-colors duration-300 delay-100">
          {open ? 'Click to collapse' : 'Click to expand'}
        </span>
        <ChevronDownIcon className={cn('size-4 transition-transform', open ? 'rotate-180' : 'rotate-0')} />
      </div>
    </DisclosureTrigger>
  )
})

export type ChainOfThoughtStepProps = ComponentProps<'div'> & {
  icon?: LucideIcon | React.FC<{ className: string }>
  label?: ReactNode
  actions?: ReactNode
  description?: ReactNode
  descriptionClassName?: string
  status?: 'complete' | 'active' | 'pending'
  isSubagent?: boolean
}

export const ChainOfThoughtStep = memo(
  ({
    className,
    icon: Icon = DotIcon,
    label,
    actions,
    description,
    descriptionClassName,
    status = 'complete',
    isSubagent = false,
    children,
    ...props
  }: ChainOfThoughtStepProps) => {
    const statusStyles = {
      active: 'text-foreground!',
      complete: 'text-muted-foreground!',
      pending: 'text-muted-foreground/50!',
    }

    return (
      <div
        className={cn(
          'flex gap-2 text-sm w-full overflow-y-hidden mb-2',
          statusStyles[status],
          'fade-in-0 slide-in-from-top-2 animate-in',
          className,
        )}
        {...props}
      >
        <div className={cn('relative mt-0.25', isSubagent && '-mt-0.25')}>
          <Icon className="size-4" />
          <div className="-mx-px absolute top-7 bottom-0 left-1/2 w-px bg-border" />
        </div>
        <div className="flex-1 space-y-2 w-full overflow-y-hidden">
          {(label || actions) && (
            <div className="flex items-center justify-between gap-2">
              {label && <div>{label}</div>}
              {actions && <div className="flex items-center gap-1">{actions}</div>}
            </div>
          )}
          {description && (
            <div className={cn('text-muted-foreground text-sm!', descriptionClassName, statusStyles[status])}>
              {description}
            </div>
          )}
          {children}
        </div>
      </div>
    )
  },
)

export type ChainOfThoughtSearchResultsProps = ComponentProps<'div'>

export const ChainOfThoughtSearchResults = memo(({ className, ...props }: ChainOfThoughtSearchResultsProps) => (
  <div className={cn('flex items-center gap-2', className)} {...props} />
))

export type ChainOfThoughtSearchResultProps = ComponentProps<typeof Badge>

export const ChainOfThoughtSearchResult = memo(({ className, children, ...props }: ChainOfThoughtSearchResultProps) => (
  <Badge className={cn('gap-1 px-2 py-0.5 font-normal text-xs', className)} variant="secondary" {...props}>
    {children}
  </Badge>
))

export type ChainOfThoughtContentProps = ComponentProps<typeof DisclosureContent> & {
  isSubagent?: boolean
}

export const ChainOfThoughtContent = memo(
  ({ className, children, isSubagent = false, ...props }: ChainOfThoughtContentProps) => {
    const { open } = useDisclosure()

    return (
      <DisclosureContent
        className={cn(
          'px-1 space-y-2',
          open && 'animate-in mt-1 pt-2 border-t border-dashed',
          !open && 'animate-out',
          open && 'fade-in-0 slide-out-to-top-2',
          !open && 'fade-out-0 slide-in-from-top-2',
          'text-popover-foreground outline-none',
          'w-full',
          !isSubagent && 'overflow-x-hidden',
          className,
        )}
        {...props}
      >
        <StickToBottom
          initial="instant"
          resize="instant"
          className={cn(
            'w-full',
            !isSubagent &&
              'overflow-y-hidden [&>div]:max-h-[300px] [&>div]:overflow-y-auto! [&>div]:overflow-x-hidden!',
            !open && 'max-h-none [&>div]:max-h-none',
          )}
        >
          <StickToBottom.Content>{children}</StickToBottom.Content>
        </StickToBottom>
      </DisclosureContent>
    )
  },
)

export type ChainOfThoughtImageProps = ComponentProps<'div'> & {
  caption?: string
}

export const ChainOfThoughtImage = memo(({ className, children, caption, ...props }: ChainOfThoughtImageProps) => (
  <div className={cn('mt-2 space-y-2', className)} {...props}>
    <div className="relative flex max-h-88 items-center justify-center overflow-hidden rounded-lg bg-muted p-3">
      {children}
    </div>
    {caption && <p className="text-muted-foreground text-xs">{caption}</p>}
  </div>
))
