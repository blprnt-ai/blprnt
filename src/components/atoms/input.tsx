import { cn } from '@/lib/utils/cn'

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {}

export const Input = ({ className, ...props }: InputProps) => {
  return (
    <input
      {...props}
      data-slot="input"
      className={cn(
        'selection:bg-primary selection:text-primary-foreground border-input',
        'h-9 w-full min-w-0 rounded-md border px-3 py-1',
        'text-base outline-none',

        'file:text-foreground placeholder:text-muted-foreground',
        'file:inline-flex file:h-7 file:border-0 file:bg-transparent file:text-sm file:font-medium',
        'disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm',
        'focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[1px]',
        'aria-invalid:ring-destructive/20 aria-invalid:border-destructive',
        'backdrop-blur-sm bg-accent',
        className,
      )}
    />
  )
}
