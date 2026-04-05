import { cva, type VariantProps } from 'class-variance-authority'
import { CheckIcon, CopyIcon } from 'lucide-react'
import { AnimatePresence, motion } from 'motion/react'
import React from 'react'
import { toast } from 'sonner'
import { ButtonPrimitive, type ButtonPrimitiveProps } from '@/components/ui/motion-button'
import { useControlledState } from '@/hooks/use-controlled-state'
import { cn } from '@/lib/utils'

const buttonVariants = cva(
  "flex items-center justify-center rounded-md transition-[box-shadow,color,background-color,border-color,outline-color,text-decoration-color,fill,stroke] disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg:not([class*='size-'])]:size-4 shrink-0 [&_svg]:shrink-0 outline-none aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive cursor-pointer",
  {
    defaultVariants: {
      size: 'default',
      variant: 'default',
    },
    variants: {
      size: {
        default: 'size-9',
        lg: 'size-10 rounded-md',
        sm: 'size-8 rounded-md',
        xs: "size-7 [&_svg:not([class*='size-'])]:size-3.5 rounded-md",
      },
      variant: {
        accent: 'bg-primary text-primary-foreground shadow-xs hover:bg-primary/90',
        default: 'bg-primary text-primary-foreground shadow-xs hover:bg-primary/90',
        destructive: 'bg-destructive text-white shadow-xs hover:bg-destructive/90 dark:bg-destructive/60',
        ghost: 'hover:bg-primary hover:text-primary-foreground dark:hover:bg-primary/50',
        link: 'text-primary underline-offset-4 hover:underline',
        outline:
          'border bg-background shadow-xs hover:bg-primary hover:text-primary-foreground dark:bg-input/30 dark:border-input dark:hover:bg-input/50',
        secondary: 'bg-accent text-secondary-foreground shadow-xs hover:bg-accent/80',
      },
    },
  },
)

type CopyButtonProps = Omit<ButtonPrimitiveProps, 'children'> &
  VariantProps<typeof buttonVariants> & {
    content: string
    copied?: boolean
    onCopiedChange?: (copied: boolean, content?: string) => void
    delay?: number
  }

export function CopyButton({
  className,
  content,
  copied,
  onCopiedChange,
  onClick,
  variant,
  size,
  delay = 3000,
  ...props
}: CopyButtonProps) {
  const [isCopied, setIsCopied] = useControlledState({
    onChange: onCopiedChange,
    value: copied,
  })

  const handleCopy = React.useCallback(
    async (e: React.MouseEvent<HTMLButtonElement>) => {
      try {
        onClick?.(e)
        if (copied || !content) return
        await writeText(content)
        setIsCopied(true)
        onCopiedChange?.(true, content)
        toast.info('Copied to clipboard', { duration: 3000 })

        setTimeout(() => {
          setIsCopied(false)
          onCopiedChange?.(false)
        }, delay)
      } catch (error) {
        console.error('Error copying command', error)
      }
    },
    [onClick, copied, content, setIsCopied, onCopiedChange, delay],
  )

  const Icon = isCopied ? CheckIcon : CopyIcon

  return (
    <ButtonPrimitive
      className={cn(buttonVariants({ className, size, variant }))}
      data-slot="copy-button"
      onClick={handleCopy}
      {...props}
    >
      <AnimatePresence mode="popLayout">
        <motion.span
          key={isCopied ? 'check' : 'copy'}
          animate={{ filter: 'blur(0px)', opacity: 1, scale: 1 }}
          data-slot="copy-button-icon"
          exit={{ filter: 'blur(4px)', opacity: 0.4, scale: 0 }}
          initial={{ filter: 'blur(4px)', opacity: 0.4, scale: 0 }}
          transition={{ duration: 0.25 }}
        >
          <Icon />
        </motion.span>
      </AnimatePresence>
    </ButtonPrimitive>
  )
}

const writeText = async (text: string) => {
  await navigator.clipboard.writeText(text)
}
