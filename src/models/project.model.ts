import { makeAutoObservable } from 'mobx'
import type { CreateProjectPayload } from '@/bindings/CreateProjectPayload'
import type { ProjectDto } from '@/bindings/ProjectDto'
import type { ProjectPatch } from '@/bindings/ProjectPatch'
import { ModelField } from './model.field'

export class ProjectModel {
  public id: string
  private _name: ModelField<string>
  private _workingDirectories: ModelField<string[]>
  public createdAt: Date
  public updatedAt: Date

  constructor(project?: ProjectDto) {
    this.id = project?.id ?? ''
    this._name = new ModelField(project?.name ?? '')
    this._workingDirectories = new ModelField(project?.working_directories ?? [])
    this.createdAt = new Date(project?.created_at ?? '')
    this.updatedAt = new Date(project?.updated_at ?? '')

    makeAutoObservable(this)
  }

  public get isValid() {
    return (
      this._name.value.length > 0 &&
      this._workingDirectories.value.length > 0 &&
      this._workingDirectories.value.every((directory) => directory.length > 0)
    )
  }

  public get isDirty() {
    return this._name.isDirty || this._workingDirectories.isDirty
  }

  public get name() {
    return this._name.value
  }

  public set name(name: string) {
    this._name.value = name
  }

  public get workingDirectories() {
    return this._workingDirectories.value
  }

  public addWorkingDirectory() {
    const workingDirectories = [...this._workingDirectories.value]
    workingDirectories.push('')

    this._workingDirectories.value = workingDirectories
  }

  public removeWorkingDirectory(index: number) {
    const workingDirectories = [...this._workingDirectories.value]
    workingDirectories.splice(index, 1)

    this._workingDirectories.value = workingDirectories
  }

  public setWorkingDirectory(index: number, workingDirectory: string) {
    const workingDirectories = [...this._workingDirectories.value]
    workingDirectories[index] = workingDirectory

    this._workingDirectories.value = workingDirectories
  }

  public toPayload(): CreateProjectPayload {
    return {
      name: this._name.value,
      working_directories: this._workingDirectories.value,
    }
  }

  public toPayloadPatch(): ProjectPatch {
    return {
      name: this._name.dirtyValue ?? undefined,
      working_directories: this._workingDirectories.dirtyValue ?? undefined,
    }
  }
}
