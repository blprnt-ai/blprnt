import { useMemo } from 'react'
import { Button } from '@/components/atoms/button'
import { Disclosure, DisclosureContent, DisclosureTrigger } from '@/components/atoms/disclosure'
import { SessionCard } from '@/components/molecules/session-card'
import { useDisclosure } from '@/hooks/use-disclosure'
import type { SessionModel } from '@/lib/models/session.model'

interface SessionSelectorProps {
  title: string
  sessions: SessionModel[]
  projectNames: Map<string, string>
  handleOpenSession: (projectId: string, sessionId: string) => void
}

export const SessionSelector = ({ title, sessions, projectNames, handleOpenSession }: SessionSelectorProps) => {
  const latestSession = useMemo(() => {
    return sessions?.[0]
  }, [sessions])

  const nextFiveSessions = useMemo(() => {
    return sessions.slice(1, 6)
  }, [sessions])

  if (!latestSession) return null

  return (
    <div className="w-2xl flex flex-col gap-4">
      <div className="text-center">
        <p className="text-xl text-muted-foreground">{title}</p>
      </div>

      <SessionCard
        projectName={projectNames.get(latestSession.projectId ?? '') ?? 'Project'}
        session={latestSession}
        onOpen={handleOpenSession}
      />

      {nextFiveSessions.length > 0 && (
        <Disclosure className="flex flex-col gap-4 w-full">
          <ViewMoreButton />
          <DisclosureContent>
            <div className="flex flex-col gap-4">
              {nextFiveSessions.map((session) => (
                <SessionCard
                  key={session.id}
                  projectName={projectNames.get(session.projectId ?? '') ?? 'Project'}
                  session={session}
                  onOpen={handleOpenSession}
                />
              ))}
            </div>
          </DisclosureContent>
        </Disclosure>
      )}
    </div>
  )
}

const ViewMoreButton = () => {
  const { open } = useDisclosure()

  return (
    <div className="flex items-center justify-center">
      <DisclosureTrigger>
        <Button variant="link">View {open ? 'less' : 'more'}</Button>
      </DisclosureTrigger>
    </div>
  )
}
