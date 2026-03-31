import { Button as ButtonPrimitive } from '@base-ui/react/button'
import { cva, type VariantProps } from 'class-variance-authority'

import { cn } from '@/lib/utils'

export const buttonVariants = cva(
  "group/button inline-flex shrink-0 items-center justify-center rounded-md border border-transparent bg-clip-padding text-sm font-medium whitespace-nowrap transition-all duration-300 outline-none select-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 active:not-aria-[haspopup]:translate-y-px disabled:pointer-events-none disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-3 aria-invalid:ring-destructive/20 dark:aria-invalid:border-destructive/50 dark:aria-invalid:ring-destructive/40 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 cursor-pointer",
  {
    defaultVariants: {
      size: 'default',
      variant: 'default',
    },
    variants: {
      size: {
        default:
          'h-9 gap-1.5 px-2.5 in-data-[slot=button-group]:rounded-md has-data-[icon=inline-end]:pr-2 has-data-[icon=inline-start]:pl-2',
        icon: 'size-9',
        'icon-lg': 'size-10',
        'icon-sm': 'size-8 rounded-[min(var(--radius-md),10px)] in-data-[slot=button-group]:rounded-md',
        'icon-xs':
          "size-6 rounded-[min(var(--radius-md),8px)] in-data-[slot=button-group]:rounded-md [&_svg:not([class*='size-'])]:size-3",
        lg: 'h-10 gap-1.5 px-2.5 has-data-[icon=inline-end]:pr-3 has-data-[icon=inline-start]:pl-3',
        sm: 'h-8 gap-1 rounded-[min(var(--radius-md),10px)] px-2.5 in-data-[slot=button-group]:rounded-md has-data-[icon=inline-end]:pr-1.5 has-data-[icon=inline-start]:pl-1.5',
        xs: "h-6 gap-1 rounded-[min(var(--radius-md),8px)] px-2 text-xs in-data-[slot=button-group]:rounded-md has-data-[icon=inline-end]:pr-1.5 has-data-[icon=inline-start]:pl-1.5 [&_svg:not([class*='size-'])]:size-3",
      },
      variant: {
        default:
          'bg-primary text-primary-foreground shadow-xs hover:bg-primary/92',
        destructive:
          'bg-destructive/12 text-destructive hover:bg-destructive/18 focus-visible:border-destructive/40 focus-visible:ring-destructive/20 dark:bg-destructive/22 dark:hover:bg-destructive/30 dark:focus-visible:ring-destructive/40',
        'destructive-ghost':
          'bg-transparent text-destructive hover:bg-destructive/14 focus-visible:ring-destructive/20 dark:hover:bg-destructive/24 dark:focus-visible:ring-destructive/40',
        'destructive-outline':
          'border-destructive/20 bg-destructive/4 text-destructive hover:bg-destructive/12 focus-visible:border-destructive/40 focus-visible:ring-destructive/20 dark:border-destructive/25 dark:hover:bg-destructive/20 dark:focus-visible:ring-destructive/40',
        ghost:
          'hover:bg-accent hover:text-accent-foreground aria-expanded:bg-accent aria-expanded:text-accent-foreground dark:hover:bg-accent/80',
        link: 'text-primary underline-offset-4 hover:underline',
        outline:
          'border-border bg-background/88 text-foreground shadow-xs hover:border-primary/25 hover:bg-accent/70 hover:text-accent-foreground aria-expanded:border-primary/25 aria-expanded:bg-accent/70 aria-expanded:text-accent-foreground dark:border-input dark:bg-background/65 dark:hover:bg-accent/70',
        secondary:
          'bg-secondary text-secondary-foreground shadow-[inset_0_1px_0_color-mix(in_oklab,var(--primary)_18%,white)] hover:bg-secondary/82 aria-expanded:bg-secondary aria-expanded:text-secondary-foreground',
      },
    },
  },
)

export const Button = ({
  className,
  variant = 'default',
  size = 'default',
  ...props
}: ButtonPrimitive.Props & VariantProps<typeof buttonVariants>) => {
  return <ButtonPrimitive className={cn(buttonVariants({ className, size, variant }))} data-slot="button" {...props} />
}
