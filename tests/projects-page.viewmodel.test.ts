import assert from 'node:assert/strict'
import test from 'node:test'

import type { ProjectDto } from '../src/bindings/ProjectDto.ts'
import { projectsApi } from '../src/lib/api/projects.ts'
import { AppModel } from '../src/models/app.model.ts'
import { ProjectViewmodel } from '../src/components/pages/project/project.viewmodel.ts'
import { ProjectsViewmodel } from '../src/components/pages/projects/projects.viewmodel.ts'

const projectFixture: ProjectDto = {
  id: 'project-1',
  name: 'Launchpad',
  working_directories: ['/Users/me/projects/launchpad', '/Users/me/projects/launchpad-api'],
  created_at: '2026-03-26T10:00:00.000Z',
  updated_at: '2026-03-27T10:00:00.000Z',
}

const secondProjectFixture: ProjectDto = {
  id: 'project-2',
  name: 'Atlas',
  working_directories: ['/Users/me/projects/atlas'],
  created_at: '2026-03-26T08:00:00.000Z',
  updated_at: '2026-03-27T08:00:00.000Z',
}

test('ProjectsViewmodel.init loads projects and syncs AppModel', async (t) => {
  const originalList = projectsApi.list
  const originalProjects = AppModel.instance.projects

  t.after(() => {
    projectsApi.list = originalList
    AppModel.instance.setProjects(originalProjects)
  })

  projectsApi.list = async () => [projectFixture, secondProjectFixture]

  const viewmodel = new ProjectsViewmodel()

  await viewmodel.init()

  assert.equal(viewmodel.projects.length, 2)
  assert.equal(viewmodel.projects[0]?.id, projectFixture.id)
  assert.equal(AppModel.instance.projects.length, 2)
  assert.equal(AppModel.instance.resolveProjectName(projectFixture.id), projectFixture.name)
})

test('ProjectViewmodel.init loads a single project into editable state', async (t) => {
  const originalGet = projectsApi.get

  t.after(() => {
    projectsApi.get = originalGet
  })

  projectsApi.get = async () => projectFixture

  const viewmodel = new ProjectViewmodel(projectFixture.id)

  await viewmodel.init()

  assert.equal(viewmodel.project?.id, projectFixture.id)
  assert.equal(viewmodel.project?.name, projectFixture.name)
  assert.equal(viewmodel.isEditing, false)
})

test('ProjectViewmodel.cancelEditing restores the original project after unsaved changes', async (t) => {
  const originalGet = projectsApi.get

  t.after(() => {
    projectsApi.get = originalGet
  })

  projectsApi.get = async () => projectFixture

  const viewmodel = new ProjectViewmodel(projectFixture.id)

  await viewmodel.init()
  viewmodel.startEditing()
  viewmodel.project!.name = 'Temporary rename'
  viewmodel.project!.setWorkingDirectory(0, '/tmp/temporary')

  viewmodel.cancelEditing()

  assert.equal(viewmodel.project?.name, projectFixture.name)
  assert.deepEqual(viewmodel.project?.workingDirectories, projectFixture.working_directories)
  assert.equal(viewmodel.isEditing, false)
})

test('ProjectViewmodel.save persists changes and upserts AppModel', async (t) => {
  const originalGet = projectsApi.get
  const originalUpdate = projectsApi.update
  const originalProjects = AppModel.instance.projects

  t.after(() => {
    projectsApi.get = originalGet
    projectsApi.update = originalUpdate
    AppModel.instance.setProjects(originalProjects)
  })

  let payload: Parameters<typeof projectsApi.update>[1] | null = null

  projectsApi.get = async () => projectFixture
  projectsApi.update = async (_id, data) => {
    payload = data

    return {
      ...projectFixture,
      name: data.name ?? projectFixture.name,
      working_directories: data.working_directories ?? projectFixture.working_directories,
    }
  }

  const viewmodel = new ProjectViewmodel(projectFixture.id)

  await viewmodel.init()
  viewmodel.startEditing()
  viewmodel.project!.name = 'Mission Control'
  viewmodel.project!.setWorkingDirectory(0, '/Users/me/projects/mission-control')

  await viewmodel.save()

  assert.equal(payload?.name, 'Mission Control')
  assert.deepEqual(payload?.working_directories, [
    '/Users/me/projects/mission-control',
    '/Users/me/projects/launchpad-api',
  ])
  assert.equal(viewmodel.project?.name, 'Mission Control')
  assert.equal(viewmodel.project?.workingDirectories[0], '/Users/me/projects/mission-control')
  assert.equal(viewmodel.isEditing, false)
  assert.equal(AppModel.instance.resolveProjectName(projectFixture.id), 'Mission Control')
})
