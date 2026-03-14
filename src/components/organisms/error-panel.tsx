import type { DockviewPanelApi } from 'dockview-react'
import { ErrorInfo } from '@/components/molecules/error-info'

interface ErrorPanelProps {
  title: string
  error: string
  panelApi: DockviewPanelApi
}
export const ErrorPanel = ({ title, error, panelApi }: ErrorPanelProps) => {
  const handleClose = () => panelApi.close()

  return <ErrorInfo action={handleClose} error={error} title={title} />
}
