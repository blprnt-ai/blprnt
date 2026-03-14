import { makeAutoObservable } from 'mobx'
import type { PersonalityModelDto as PersonalityDto } from '@/bindings'
import { tauriPersonalitiesApi } from '@/lib/api/tauri/personalities.api'

export interface PersonalitySnapshot {
  id: string
  name: string
  description: string
  systemPrompt: string
  isDefault: boolean
  isUserDefined: boolean
  createdAt: string
  updatedAt: string
}

export class PersonalityModel {
  public id: string
  public name: string
  public description: string
  public systemPrompt: string
  public isDefault: boolean
  public isUserDefined: boolean
  public createdAt: string
  public updatedAt: string

  constructor(model: PersonalityDto) {
    this.id = model.id
    this.name = model.name
    this.description = model.description
    this.systemPrompt = model.system_prompt
    this.isDefault = model.is_default
    this.isUserDefined = model.is_user_defined
    this.createdAt = model.created_at
    this.updatedAt = model.updated_at

    makeAutoObservable(this, {}, { autoBind: true })
  }

  updateFrom = (model: PersonalityDto) => {
    this.name = model.name
    this.description = model.description
    this.systemPrompt = model.system_prompt
    this.isDefault = model.is_default
    this.isUserDefined = model.is_user_defined
    this.createdAt = model.created_at
    this.updatedAt = model.updated_at
  }

  static list = async () => {
    const result = await tauriPersonalitiesApi.list()
    return result.map((item) => new PersonalityModel(item))
  }

  static create = async (name: string, description: string, systemPrompt: string) => {
    const result = await tauriPersonalitiesApi.create(name, description, systemPrompt)
    return new PersonalityModel(result)
  }

  update = async (patch: Partial<Pick<PersonalityModel, 'name' | 'description' | 'systemPrompt'>>) => {
    const nextName = patch.name ?? this.name
    const nextDescription = patch.description ?? this.description
    const nextPrompt = patch.systemPrompt ?? this.systemPrompt

    const result = await tauriPersonalitiesApi.update(this.id, nextName, nextDescription, nextPrompt)
    this.updateFrom(result)
    return this
  }

  delete = async () => {
    await tauriPersonalitiesApi.delete(this.id)
  }
}
