import { Slot } from '@radix-ui/react-slot'
import { cva, type VariantProps } from 'class-variance-authority'
import type React from 'react'

import { cn } from '@/lib/utils/cn'

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium transition-all disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg:not([class*='size-'])]:size-4 shrink-0 [&_svg]:shrink-0 aria-invalid:ring-destructive/20 aria-invalid:border-destructive cursor-pointer duration-300 focus-visible:ring-1 focus-visible:ring-amber-500 focus-visible:border-amber-500",
  {
    defaultVariants: {
      size: 'default',
      variant: 'default',
    },
    variants: {
      size: {
        default: 'h-9 px-4 py-2 has-[>svg]:px-3',
        icon: 'size-9',
        'icon-lg': 'size-10',
        'icon-sm': 'size-8',
        'icon-xs': 'size-7',
        lg: 'h-10 rounded-md px-6 has-[>svg]:px-4',
        sm: 'h-8 rounded-md gap-1.5 px-3 has-[>svg]:px-2.5',
        xl: 'h-12 rounded-md px-8 has-[>svg]:px-6 text-xl',
        xs: "size-7 [&_svg:not([class*='size-'])]:size-3.5 rounded-md",
      },
      variant: {
        circle: 'rounded-full border-border',
        default: 'bg-primary text-primary-foreground hover:bg-primary/60',
        destructive: 'bg-destructive/30 hover:bg-destructive/70',
        'destructive-ghost': 'hover:bg-primary hover:text-destructive/70!',
        ghost: 'hover:bg-primary hover:text-primary-foreground',
        link: 'text-primary underline-offset-4 hover:underline',
        outline: 'border border-primary/60 shadow-xs hover:bg-primary/30 backdrop-blur-[2px]',
        'outline-ghost': 'border border-transparent hover:border-primary/60',
        rounded: 'rounded-2xl border-border',
        secondary: 'bg-accent text-secondary-foreground hover:bg-accent/80',
      },
    },
  },
)

export type ButtonProps = React.ComponentProps<'button'> &
  VariantProps<typeof buttonVariants> & {
    asChild?: boolean
  }

export const Button = ({ className, variant, size, asChild = false, ...props }: ButtonProps) => {
  const Comp = asChild ? Slot : 'button'

  return <Comp className={cn(buttonVariants({ className, size, variant }))} data-slot="button" {...props} />
}
