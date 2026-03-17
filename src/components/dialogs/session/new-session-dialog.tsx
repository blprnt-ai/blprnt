import { useEffect, useMemo, useState } from 'react'
import type { TauriError } from '@/bindings'
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/atoms/dialog'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/atoms/select'
import { newSessionToast as toast } from '@/components/atoms/toaster'
import { SessionForm } from '@/components/forms/session/session-form'
import { SessionFormViewModel } from '@/components/forms/session/session-form.viewmodel'
import { ProjectModel } from '@/lib/models/project.model'
import { SessionModel } from '@/lib/models/session.model'
import { defaultSessionModel } from '@/lib/utils/default-models'
import { reportError } from '@/lib/utils/error-reporting'
import { errorToMessage } from '@/lib/utils/misc'

interface NewSessionDialogProps {
  initialProjectId?: string
  isOpen: boolean
  onOpenChange: (open: boolean) => void
  onAfterCreate?: (session: SessionModel) => void
}

export const NewSessionDialog = ({ initialProjectId, isOpen, onOpenChange, onAfterCreate }: NewSessionDialogProps) => {
  const [newSession] = useState<SessionFormViewModel>(
    () => new SessionFormViewModel(new SessionModel({ ...defaultSessionModel, queue_mode: 'queue', status: 'Idle' })),
  )

  const [selectedProjectId, setSelectedProjectId] = useState<string | undefined>(initialProjectId)
  const [projects, setProjects] = useState<ProjectModel[]>([])

  const handleProjectSelect = (value: string) => {
    setSelectedProjectId(value)
  }

  useEffect(() => {
    if (initialProjectId || !projects.length) return

    setSelectedProjectId(projects[0].id)
  }, [initialProjectId, projects])

  const handleSubmit = async () => {
    if (!newSession.isValid || !selectedProjectId) return

    try {
      toast.loading({ title: 'Creating session...' })
      const session = await SessionModel.create(newSession.toCreateParams(selectedProjectId))
      onAfterCreate?.(session)
      toast.success({ title: 'Session created successfully' })
      onOpenChange(false)
    } catch (error) {
      reportError(error, 'creating session')
      toast.error(errorToMessage(error as TauriError))
    }
  }

  const handleOpenChange = (open: boolean) => {
    onOpenChange(open)
  }

  useEffect(() => {
    let isMounted = true
    ProjectModel.list()
      .then((list) => {
        if (!isMounted) return
        setProjects(list)
      })
      .catch((error) => {
        console.error('Error loading projects', error)
      })

    return () => {
      isMounted = false
    }
  }, [])

  return (
    <Dialog open={isOpen} onOpenChange={handleOpenChange}>
      <DialogContent size="md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-1.5">
            {initialProjectId || !projects.length || projects.length === 1 ? (
              <div>New Session</div>
            ) : (
              <>
                <div>New Session for</div>
                <ProjectSelector projectId={selectedProjectId} projects={projects} onSelect={handleProjectSelect} />
              </>
            )}
          </DialogTitle>
          <DialogDescription>Configure and start a new blprnt session</DialogDescription>
        </DialogHeader>

        <SessionForm isNew session={newSession} onSubmit={handleSubmit} />
      </DialogContent>
    </Dialog>
  )
}

const ProjectSelector = ({
  projectId,
  projects,
  onSelect,
}: {
  projectId?: string
  projects: ProjectModel[]
  onSelect: (value: string) => void
}) => {
  const project = useMemo(() => projects.find((item) => item.id === projectId), [projects, projectId])
  const projectName = useMemo(() => project?.name, [project])

  return (
    <Select value={String(projectId)} onValueChange={onSelect}>
      <SelectTrigger
        chevronClassName="text-primary"
        className="border-0 text-primary hover:bg-none transition-colors duration-300 text-[20px] p-0! font-medium!"
        onKeyDown={() => {}}
      >
        <SelectValue>{projectName}</SelectValue>
      </SelectTrigger>
      <SelectContent>
        {projects.map((project) => (
          <SelectItem key={project.id} value={project.id}>
            {project.name}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  )
}
