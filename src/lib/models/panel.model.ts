import { makeAutoObservable } from 'mobx'

export type PanelId = string

export enum PanelType {
  Session = 'session',
  Project = 'project',
  Intro = 'intro',
  Personality = 'personality',
  UserAccount = 'user-account',
  Preview = 'preview',
  Plan = 'plan',
}

export interface PanelSnapshot {
  id: PanelId
  type: PanelType
  title: string
  params?: Record<string, unknown>
  isActive?: boolean
  isVisible?: boolean
}

export interface PanelPatch {
  title?: string
  params?: Record<string, unknown>
  isActive?: boolean
  isVisible?: boolean
}

export class PanelModel {
  public id: PanelId
  public type: PanelType
  public title: string
  public params: Record<string, unknown>
  public isActive: boolean
  public isVisible: boolean

  constructor(snapshot: PanelSnapshot) {
    this.id = snapshot.id
    this.type = snapshot.type
    this.title = snapshot.title
    this.params = snapshot.params ?? {}
    this.isActive = snapshot.isActive ?? true
    this.isVisible = snapshot.isVisible ?? false

    makeAutoObservable(this, {}, { autoBind: true })
  }

  updateFrom = (snapshot: PanelSnapshot) => {
    this.type = snapshot.type
    this.title = snapshot.title
    this.params = snapshot.params ?? {}
    this.isActive = snapshot.isActive ?? this.isActive
    this.isVisible = snapshot.isVisible ?? this.isVisible
  }

  static list = (snapshots: PanelSnapshot[]) => {
    return snapshots.map((snapshot) => new PanelModel(snapshot))
  }

  static create = (snapshot: PanelSnapshot) => {
    return new PanelModel(snapshot)
  }

  update = (patch: PanelPatch) => {
    if (patch.title !== undefined) this.title = patch.title
    if (patch.params !== undefined) this.params = patch.params
    if (patch.isActive !== undefined) this.isActive = patch.isActive
    if (patch.isVisible !== undefined) this.isVisible = patch.isVisible

    return this
  }

  delete = () => {
    return true
  }
}
