import { createContext, type PropsWithChildren, useContext, useMemo } from 'react'
import type { ProjectModel } from '@/lib/models/project.model'
import { ProjectTreeViewmodel } from './project-tree.viewmodel'

const ProjectTreeViewmodelContext = createContext<ProjectTreeViewmodel | null>(null)

export const useProjectTreeViewmodel = () => {
  const viewmodel = useContext(ProjectTreeViewmodelContext)
  if (!viewmodel) throw new Error('useProjectTreeViewmodel must be used within ProjectTreeProvider')

  return viewmodel
}

interface ProjectTreeProviderProps extends PropsWithChildren {
  project: ProjectModel
}

export const ProjectTreeProvider = ({ project, children }: ProjectTreeProviderProps) => {
  const viewmodel = useMemo(() => new ProjectTreeViewmodel(project), [project])

  return <ProjectTreeViewmodelContext.Provider value={viewmodel}>{children}</ProjectTreeViewmodelContext.Provider>
}
