import { homeDir } from '@tauri-apps/api/path'
import { stat } from '@tauri-apps/plugin-fs'
import { debounce } from 'lodash'
import { flow, makeAutoObservable, reaction, toJS } from 'mobx'
import { createContext, useContext } from 'react'
import type { ProjectPatchV2 } from '@/bindings'
import { ProjectModel } from '@/lib/models/project.model'
import { defaultProjectModel, newProjectId } from '@/lib/utils/default-models'

const placeHolderAgentPrimer = `# Agent Primer

---

### You are a specialized agent for this project.

Your goals are to:

- Do the work
- Use the tools provided to you
- Be concise and to the point
`

export class ProjectEditorViewModel {
  public model: ProjectModel | null = null
  public homeDirectory: string | null = null
  public isValidWorkingDirectories: boolean[] = []
  public disposers: Array<() => void> = []
  public autoSaveEnabled: boolean = true
  private autoSave = false
  private isDirty = false

  constructor(private readonly projectId: string) {
    makeAutoObservable(this, { autoSaveEnabled: false, disposers: false, model: false }, { autoBind: true })
  }

  init = flow(function* (this: ProjectEditorViewModel, autoSave: boolean = false) {
    this.autoSave = autoSave
    this.homeDirectory = yield homeDir()
    this.model =
      this.projectId === newProjectId ? new ProjectModel(defaultProjectModel) : yield ProjectModel.get(this.projectId)

    this.validateWorkingDirectories()

    this.maybeInitReactions()
  })

  destroy = () => {
    this.disposers.forEach((disposer) => disposer())
    this.disposers = []
  }

  maybeInitReactions = () => {
    console.log('maybeInitReactions', this.autoSave)
    if (this.autoSave) {
      this.disposers.push(
        reaction(
          () => this.name,
          () => this.save(),
        ),
      )

      this.disposers.push(
        reaction(
          () => this.agentPrimer,
          () => this.save(),
        ),
      )

      this.disposers.push(
        reaction(
          () => this.workingDirectories.slice(),
          () => {
            console.log('workingDirectories changed')
            this.save()
          },
        ),
      )
    }
  }

  save = debounce(async () => {
    console.log('this.isDirty', this.isDirty)
    if (!this.isDirty) return

    try {
      console.log('saving1')
      this.destroy()
      console.log('saving2')
      await this.update()
      console.log('saving3')
      this.maybeInitReactions()
    } catch {}
  }, 1000)

  get id() {
    return this.model!.id
  }

  get name() {
    return this.model?.name ?? ''
  }

  setName = (name: string) => {
    this.model!.name = name
    this.isDirty = true
  }

  get workingDirectories() {
    return this.model!.workingDirectories
  }

  get hasWorkingDirectories() {
    return this.workingDirectories.length > 0
  }

  addWorkingDirectory = () => {
    const workingDirectory = this.homeDirectory ?? ''

    this.model!.workingDirectories.push(workingDirectory)
    this.validateWorkingDirectories()
    this.isDirty = true
  }

  removeWorkingDirectory = (index: number) => {
    this.model!.workingDirectories.splice(index, 1)
    this.validateWorkingDirectories()
    this.isDirty = true
  }

  changeWorkingDirectory = (index: number, workingDirectory: string) => {
    this.model!.workingDirectories[index] = workingDirectory
    this.validateWorkingDirectories()
    this.isDirty = true
  }

  validateWorkingDirectories = flow(function* (this: ProjectEditorViewModel) {
    const isValid = yield Promise.all(
      this.workingDirectories.map(async (workingDirectory) => {
        try {
          const result = await stat(workingDirectory)
          return result.isDirectory
        } catch {
          return false
        }
      }),
    )

    this.isValidWorkingDirectories = isValid
  })

  get placeHolderAgentPrimer() {
    return placeHolderAgentPrimer
  }

  get agentPrimer() {
    return this.model!.agentPrimer ?? ''
  }

  setAgentPrimer = (agentPrimer: string) => {
    this.model!.agentPrimer = agentPrimer
    this.isDirty = true
  }

  get createdAt() {
    return this.model!.createdAt
  }

  get updatedAt() {
    return this.model!.updatedAt
  }

  get isValid() {
    return (
      this.name.trim().length >= 1 &&
      this.hasWorkingDirectories &&
      this.isValidWorkingDirectories.every((isValid) => isValid)
    )
  }

  update = flow(function* (this: ProjectEditorViewModel) {
    console.log('update1', this.isValid)
    if (!this.isValid) return

    const projectPatch: ProjectPatchV2 = {
      agent_primer: this.agentPrimer,
      name: this.name,
      working_directories: this.workingDirectories,
    }
    if (Object.keys(projectPatch).length === 0) return

    yield this.model!.update(toJS(projectPatch))
    this.isDirty = false
  })

  create = flow(async function* (this: ProjectEditorViewModel) {
    const model = yield ProjectModel.create({
      agentPrimer: this.agentPrimer,
      name: this.name,
      workingDirectories: this.workingDirectories,
    })
    this.isDirty = false
    return model
  })

  delete = flow(function* (this: ProjectEditorViewModel) {
    yield this.model!.delete()
  })
}

export const ProjectEditorViewModelContext = createContext<ProjectEditorViewModel | null>(null)
export const useProjectEditorViewModel = () => {
  const viewmodel = useContext(ProjectEditorViewModelContext)
  if (!viewmodel) throw new Error('useProjectEditorViewModel must be used within ProjectEditorViewModelContext')

  return viewmodel
}
