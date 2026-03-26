import { Input as InputPrimitive } from '@base-ui/react/input'
import { cva, type VariantProps } from 'class-variance-authority'
import type * as React from 'react'
import { cn } from '@/lib/utils'

export const inputVariants = cva(
  [
    'w-full min-w-0 font-light',
    'rounded-md border border-input outline-none',
    'bg-input/5 shadow-xs',
    'transition-[color,box-shadow]',
    'file:inline-flex file:h-7 file:border-0 file:bg-transparent file:text-sm file:font-medium file:text-foreground',
    'placeholder:text-muted-foreground',
    'focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50',
    'disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:border-destructive',
    'aria-invalid:ring-3 aria-invalid:ring-destructive/20',
    'dark:bg-input/30 dark:aria-invalid:border-destructive/50 dark:aria-invalid:ring-destructive/40',
  ],
  {
    defaultVariants: {
      size: 'md',
    },
    variants: {
      size: {
        md: 'h-9 px-2.5 py-1 text-base md:text-sm',
        sm: 'h-7 px-2 py-1 text-xs',
      },
    },
  },
)

interface InputProps extends Omit<React.ComponentProps<'input'>, 'size'> {
  size?: VariantProps<typeof inputVariants>['size']
}

export const Input = ({ className, type, size, ...props }: InputProps) => {
  return <InputPrimitive className={cn(inputVariants({ className, size }))} data-slot="input" type={type} {...props} />
}
