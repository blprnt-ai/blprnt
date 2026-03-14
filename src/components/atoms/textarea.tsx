import { cva, type VariantProps } from 'class-variance-authority'
import TextareaAutosize, { type TextareaAutosizeProps } from 'react-textarea-autosize'
import { cn } from '@/lib/utils/cn'

const textareaVariants = cva(
  'border-input placeholder:text-muted-foreground aria-invalid:ring-destructive/20 aria-invalid:border-destructive flex field-sizing-content min-h-8 w-full rounded-md border bg-accent px-4 py-2 text-base outline-none disabled:cursor-not-allowed md:text-sm',
  {
    defaultVariants: {
      variant: 'default',
    },
    variants: {
      variant: {
        default: 'focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[1px]',
        'no-focus': 'focus:border focus-visible:ring-0',
      },
    },
  },
)

interface TextareaProps extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  variant?: VariantProps<typeof textareaVariants>['variant']
  ref?: React.Ref<HTMLTextAreaElement>
}

export const Textarea = ({ className, variant, style, ...props }: TextareaProps) => {
  return (
    <TextareaAutosize
      {...props}
      className={cn(textareaVariants({ variant }), className)}
      data-slot="textarea"
      style={style as unknown as TextareaAutosizeProps['style']}
    />
  )
}
