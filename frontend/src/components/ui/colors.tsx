import { cva, type VariantProps } from 'class-variance-authority'
import { cn } from '@/lib/utils'

const bgColorVariants = cva('', {
  variants: {
    color: {
      amber: 'bg-amber-500 dark:bg-amber-700',
      blue: 'bg-blue-600 dark:bg-blue-700',
      emerald: 'bg-emerald-700 dark:bg-emerald-800',
      fuchsia: 'bg-fuchsia-500 dark:bg-fuchsia-700',
      gray: 'bg-gray-500 dark:bg-gray-300',
      green: 'bg-green-500 dark:bg-green-700',
      orange: 'bg-orange-400 dark:bg-orange-500',
      pink: 'bg-pink-500 dark:bg-pink-500',
      purple: 'bg-purple-500 dark:bg-purple-700',
      red: 'bg-red-500 dark:bg-red-700',
      yellow: 'bg-yellow-400 dark:bg-yellow-500',
    },
  },
})

const textColorVariants = cva('', {
  variants: {
    color: {
      amber: 'text-amber-500 dark:text-amber-700',
      blue: 'text-blue-600 dark:text-blue-700',
      emerald: 'text-emerald-700 dark:text-emerald-800',
      fuchsia: 'text-fuchsia-500 dark:text-fuchsia-700',
      gray: 'text-gray-500 dark:text-gray-300',
      green: 'text-green-500 dark:text-green-700',
      orange: 'text-orange-400 dark:text-orange-500',
      pink: 'text-pink-500 dark:text-pink-500',
      purple: 'text-purple-500 dark:text-purple-700',
      red: 'text-red-500 dark:text-red-700',
      yellow: 'text-yellow-400 dark:text-yellow-500',
    },
  },
})

export type ColorVariant = Exclude<VariantProps<typeof bgColorVariants>['color'], null | undefined>

export const colors: { color: ColorVariant; name: string; default?: boolean }[] = [
  { color: 'red', name: 'Red' },
  { color: 'amber', name: 'Amber' },
  { color: 'orange', name: 'Orange' },
  { color: 'yellow', name: 'Yellow' },
  { color: 'green', name: 'Green' },
  { color: 'emerald', name: 'Emerald' },
  { color: 'blue', name: 'Blue' },
  { color: 'pink', name: 'Pink' },
  { color: 'fuchsia', name: 'Fuchsia' },
  { color: 'purple', name: 'Purple' },
  { color: 'gray', default: true, name: 'Gray' },
]

export const fallbackColor = colors.find((color) => color.default)?.color ?? colors[0].color

export const ColoredSpan = ({
  color,
  className,
  children,
}: {
  color: ColorVariant
  className?: string
  children?: React.ReactNode
}) => {
  return <span className={cn(bgColorVariants({ color }), className)}>{children}</span>
}

export const TextColoredSpan = ({
  color,
  className,
  children,
}: {
  color: ColorVariant
  className?: string
  children?: React.ReactNode
}) => {
  return <span className={cn(textColorVariants({ color }), className)}>{children}</span>
}
