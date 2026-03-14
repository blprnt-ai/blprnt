import { makeAutoObservable } from 'mobx'
import type { ProjectModel } from '@/lib/models/project.model'

export class ProjectTreeViewmodel {
  constructor(public readonly project: ProjectModel) {
    makeAutoObservable(this, { project: false }, { autoBind: true })
  }
}
