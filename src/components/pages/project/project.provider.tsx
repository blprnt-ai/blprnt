import { useParams } from '@tanstack/react-router'
import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { AppLoader } from '@/components/organisms/app-loader'
import { ProjectPage } from './project.page'
import { ProjectViewmodel, ProjectViewmodelContext } from './project.viewmodel'

export const ProjectProvider = observer(() => {
  const { projectId } = useParams({ from: '/projects/$projectId/' })
  const [viewmodel, setViewmodel] = useState(() => new ProjectViewmodel(projectId))

  useEffect(() => {
    const nextViewmodel = new ProjectViewmodel(projectId)
    setViewmodel(nextViewmodel)
    void nextViewmodel.init()

    return () => {
      nextViewmodel.destroy()
    }
  }, [projectId])

  if (viewmodel.isLoading) return <AppLoader />

  return (
    <ProjectViewmodelContext.Provider value={viewmodel}>
      <ProjectPage />
    </ProjectViewmodelContext.Provider>
  )
})
