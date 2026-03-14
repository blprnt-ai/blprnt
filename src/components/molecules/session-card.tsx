import { useMemo } from 'react'
import { GlassCard } from '@/components/atoms/glass-card'
import type { SessionModel } from '@/lib/models/session.model'
import { toHumanTime } from '@/lib/utils/misc'

interface SessionCardProps {
  session: SessionModel
  projectName: string
  onOpen: (projectId: string, sessionId: string) => void
}

export const SessionCard = ({ session, onOpen, projectName }: SessionCardProps) => {
  const title = useMemo(() => projectName || 'Project', [projectName])

  const subtitle = useMemo(() => {
    return session.name || 'New Session'
  }, [session.name])

  const handleClick = () => {
    if (!session.projectId) return
    onOpen(session.projectId, session.id)
  }

  return (
    <GlassCard className="w-full p-2 cursor-pointer" onClick={handleClick}>
      <div className="p-2">
        <div className="flex items-center justify-between">
          <div className="text-base font-medium">{title}</div>
          <p className="text-sm text-muted-foreground">
            last used {toHumanTime(session.updatedAt ?? session.createdAt)}
          </p>
        </div>

        <div className="mt-2 italic whitespace-nowrap text-ellipsis overflow-hidden text-muted-foreground text-sm">
          {subtitle}
        </div>
      </div>
    </GlassCard>
  )
}
