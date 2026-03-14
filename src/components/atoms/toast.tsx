import { cva, type VariantProps } from 'class-variance-authority'
import { XIcon } from 'lucide-react'
import { type ToasterProps, toast } from 'sonner'
import { cn } from '@/lib/utils/cn'
import { isLinux } from '@/lib/utils/is-linux'

const toastVariants = cva(
  'flex rounded-lg bg-background shadow-lg ring-1 w-full items-center p-4 transition-all duration-300 backdrop-blur-md',
  {
    variants: {
      variant: {
        error: cn('bg-red-900/20 ring-red-700/60', isLinux && 'bg-red-900/90'),
        info: cn('bg-blue-900/20 ring-blue-800/60', isLinux && 'bg-blue-900/90'),
        loading: cn('bg-white/10 text-foreground ring-primary/40', isLinux && 'bg-background/95'),
        success: cn('bg-green-900/20 ring-green-700/60', isLinux && 'bg-green-900/90'),
        warning: cn('bg-yellow-900/20 ring-yellow-700/60', isLinux && 'bg-yellow-900/90'),
      },
    },
  },
)

export interface ToastProps {
  id?: string | number
  size?: 'default' | 'wide'
  duration?: number
  dismissible?: boolean
  title: string
  position?: ToasterProps['position']
  description?: React.ReactNode
}

export type ToastComponentProps = ToastProps &
  VariantProps<typeof toastVariants> & {
    icon: React.ReactNode
  }

export const Toast = ({ id, title, description, icon, variant, dismissible }: ToastComponentProps) => {
  return (
    <div className={toastVariants({ variant })}>
      <div className="flex flex-1 items-center w-full">
        <div className="w-full">
          <div className="flex gap-2 items-center justify-between">
            <div className="flex gap-2 items-center">
              <div>{icon}</div>
              <div className="text-sm font-medium">{title}</div>
            </div>

            {dismissible && (
              <div>
                <XIcon className="cursor-pointer" size={16} onClick={() => toast.dismiss(id)} />
              </div>
            )}
          </div>
          {description && <div className="mt-1 text-sm">{description}</div>}
        </div>
      </div>
    </div>
  )
}
