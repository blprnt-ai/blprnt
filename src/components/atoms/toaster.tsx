import { CircleCheckIcon, InfoIcon, Loader2Icon, TriangleAlertIcon } from 'lucide-react'
import { toast as sonnerToast } from 'sonner'
import { Toast, type ToastComponentProps, type ToastProps } from './toast'

enum ToastId {
  NewProject = 'new-project',
  EditProject = 'edit-project',
  DeleteProject = 'delete-project',
  ImportExportProject = 'import-export-project',

  NewProvider = 'new-provider',
  DeleteProvider = 'delete-provider',

  NewSession = 'new-session',
  EditSession = 'edit-session',
  StopSession = 'stop-session',
  DeleteSession = 'delete-session',

  NewPersonality = 'new-personality',
  EditPersonality = 'edit-personality',
  DeletePersonality = 'delete-personality',

  Update = 'update',
}

type SansIdToastProps = Omit<ToastProps, 'id'>

const makeCustomToast = (id: ToastId) => {
  return {
    error: (props: SansIdToastProps) => basicToast.error({ ...props, id }),
    info: (props: SansIdToastProps) => basicToast.info({ ...props, id }),
    loading: (props: SansIdToastProps) => basicToast.loading({ ...props, id }),
    success: (props: SansIdToastProps) => basicToast.success({ ...props, id }),
    warning: (props: SansIdToastProps) => basicToast.warning({ ...props, id }),
  }
}

export const newProjectToast = makeCustomToast(ToastId.NewProject)
export const deleteProjectToast = makeCustomToast(ToastId.DeleteProject)
export const editProjectToast = makeCustomToast(ToastId.EditProject)
export const importExportProjectToast = makeCustomToast(ToastId.ImportExportProject)

export const newProviderToast = makeCustomToast(ToastId.NewProvider)
export const deleteProviderToast = makeCustomToast(ToastId.DeleteProvider)

export const newSessionToast = makeCustomToast(ToastId.NewSession)
export const editSessionToast = makeCustomToast(ToastId.EditSession)
export const stopSessionToast = makeCustomToast(ToastId.StopSession)
export const deleteSessionToast = makeCustomToast(ToastId.DeleteSession)

export const newPersonalityToast = makeCustomToast(ToastId.NewPersonality)
export const editPersonalityToast = makeCustomToast(ToastId.EditPersonality)
export const deletePersonalityToast = makeCustomToast(ToastId.DeletePersonality)

export const updateToast = makeCustomToast(ToastId.Update)

export const basicToast = {
  error: (props: ToastProps) => {
    customToast({
      ...props,
      duration: 30000,
      icon: <TriangleAlertIcon className="size-4 text-red-500" />,
      variant: 'error',
    })

    return false as const
  },
  info: (props: ToastProps) => {
    customToast({ ...props, icon: <InfoIcon className="size-4" />, variant: 'info' })
  },
  loading: (props: ToastProps) => {
    customToast({
      ...props,
      icon: <Loader2Icon className="size-4 animate-spin text-primary/60" />,
      variant: 'loading',
    })
  },
  success: (props: ToastProps) => {
    customToast({ ...props, icon: <CircleCheckIcon className="size-4 text-green-500" />, variant: 'success' })
  },
  warning: (props: ToastProps) => {
    customToast({ ...props, icon: <TriangleAlertIcon className="size-4 text-yellow-500" />, variant: 'warning' })
  },
}

const customToast = ({ id, size = 'default', ...props }: ToastComponentProps) => {
  const { title, description, icon, variant } = props

  const dismissible = variant === 'loading' ? false : (props.dismissible ?? true)
  const duration = variant === 'loading' ? Infinity : (props.duration ?? 5000)
  const position = props.position ?? 'top-right'

  const toastId = id ?? crypto.randomUUID()

  return sonnerToast.custom(
    () => (
      <Toast
        description={description}
        dismissible={dismissible}
        icon={icon}
        id={toastId}
        title={title}
        variant={variant}
      />
    ),
    {
      dismissible,
      duration,
      id: toastId,
      position,
      style: {
        width: size === 'wide' ? '500px' : '100%',
      },
    },
  )
}

export const providerNotFoundToast = (sessionName: string) => {
  return basicToast.warning({
    description: (
      <div className="flex flex-col gap-2">
        <p>
          The authentication provider for <span className="font-medium">{sessionName}</span> was not found.
        </p>
        <p>Please update the session settings.</p>
      </div>
    ),
    title: 'Authentication provider not found',
  })
}
