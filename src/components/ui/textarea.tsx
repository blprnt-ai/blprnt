import type * as React from 'react'
import TextareaAutosize, { type TextareaAutosizeProps } from 'react-textarea-autosize'

import { cn } from '@/lib/utils'

type TextareaProps = React.ComponentProps<'textarea'> &
  TextareaAutosizeProps & {
    ref?: React.Ref<HTMLTextAreaElement>
  }

export const Textarea = ({ className, style, ...props }: TextareaProps) => {
  return (
    <TextareaAutosize
      data-slot="textarea"
      style={style as unknown as TextareaAutosizeProps['style']}
      className={cn(
        'flex px-2.5 py-2 field-sizing-content min-h-16 w-full',
        'font-light rounded-md border border-input',
        'text-base md:text-sm',
        'bg-background/88 shadow-xs',
        'transition-[color,box-shadow,border-color,background-color] outline-none',
        'placeholder:text-muted-foreground',
        'hover:border-primary/20 hover:bg-muted/40',
        'focus-visible:border-ring focus-visible:bg-background focus-visible:ring-3 focus-visible:ring-ring/45',
        'disabled:cursor-not-allowed disabled:opacity-50',
        'aria-invalid:border-destructive aria-invalid:ring-3 aria-invalid:ring-destructive/20',
        'dark:bg-input/28 dark:hover:bg-muted/50 dark:aria-invalid:border-destructive/50 dark:aria-invalid:ring-destructive/40',
        className,
      )}
      {...props}
    />
  )
}
