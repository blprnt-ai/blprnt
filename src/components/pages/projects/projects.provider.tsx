import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { AppLoader } from '@/components/organisms/app-loader'
import { ProjectsPage } from './projects.page'
import { ProjectsViewmodel, ProjectsViewmodelContext } from './projects.viewmodel'

export const ProjectsProvider = observer(() => {
  const [viewmodel] = useState(() => new ProjectsViewmodel())

  useEffect(() => {
    void viewmodel.init()
  }, [viewmodel])

  if (viewmodel.isLoading) return <AppLoader />

  return (
    <ProjectsViewmodelContext.Provider value={viewmodel}>
      <ProjectsPage />
    </ProjectsViewmodelContext.Provider>
  )
})
