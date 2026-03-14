import { cn } from '@/lib/utils/cn'

export const Kbd = ({ className, ...props }: React.ComponentProps<'kbd'>) => {
  return (
    <kbd
      data-slot="kbd"
      className={cn(
        'bg-muted text-muted-foreground pointer-events-none inline-flex h-5 w-fit min-w-5 items-center justify-center gap-1 rounded-sm px-1 font-sans text-xs font-medium select-none',
        "[&_svg:not([class*='size-'])]:size-3",
        'in-data-[slot=tooltip-content]:bg-background/20 in-data-[slot=tooltip-content]:text-background',
        className,
      )}
      {...props}
    />
  )
}

export const KbdGroup = ({ className, ...props }: React.ComponentProps<'div'>) => {
  return <kbd className={cn('inline-flex items-center gap-1', className)} data-slot="kbd-group" {...props} />
}
