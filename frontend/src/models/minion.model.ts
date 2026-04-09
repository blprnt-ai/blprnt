import { makeAutoObservable } from 'mobx'
import type { CreateMinionPayload } from '@/bindings/CreateMinionPayload'
import type { MinionDto } from '@/bindings/MinionDto'
import type { MinionPatchPayload } from '@/bindings/MinionPatchPayload'
import type { MinionSource } from '@/bindings/MinionSource'
import { ModelField } from './model-field'

export class MinionModel {
  public id: string
  public source: MinionSource
  public canEditDefinition: boolean
  public canToggleEnabled: boolean
  public createdAt: Date
  public updatedAt: Date
  private _slug: ModelField<string>
  private _displayName: ModelField<string>
  private _description: ModelField<string>
  private _enabled: ModelField<boolean>
  private _prompt: ModelField<string>

  constructor(minion?: MinionDto) {
    this.id = minion?.id ?? ''
    this.source = minion?.source ?? 'custom'
    this.canEditDefinition = minion?.can_edit_definition ?? true
    this.canToggleEnabled = minion?.can_toggle_enabled ?? true
    this.createdAt = new Date(minion?.created_at ?? '')
    this.updatedAt = new Date(minion?.updated_at ?? '')
    this._slug = new ModelField(minion?.slug ?? '')
    this._displayName = new ModelField(minion?.display_name ?? '')
    this._description = new ModelField(minion?.description ?? '')
    this._enabled = new ModelField(minion?.enabled ?? true)
    this._prompt = new ModelField(minion?.prompt ?? '')

    makeAutoObservable(this)
  }

  public get isNew() {
    return this.id.length === 0
  }

  public get isSystem() {
    return this.source === 'system'
  }

  public get isReadOnly() {
    return !this.canEditDefinition && !this.canToggleEnabled
  }

  public get isDefinitionReadOnly() {
    return !this.canEditDefinition
  }

  public get isToggleReadOnly() {
    return !this.canToggleEnabled
  }

  public get isValid() {
    if (this.isNew || this.canEditDefinition) {
      if (this.slug.trim().length === 0) return false
      if (this.displayName.trim().length === 0) return false
      if (this.description.trim().length === 0) return false
    }

    if (!this.canEditDefinition) return true

    return this.prompt.trim().length > 0
  }

  public get isDirty() {
    return (
      this._slug.isDirty ||
      this._displayName.isDirty ||
      this._description.isDirty ||
      this._enabled.isDirty ||
      this._prompt.isDirty
    )
  }

  public get slug() {
    return this._slug.value
  }

  public set slug(value: string) {
    this._slug.value = value
  }

  public get displayName() {
    return this._displayName.value
  }

  public set displayName(value: string) {
    this._displayName.value = value
  }

  public get description() {
    return this._description.value
  }

  public set description(value: string) {
    this._description.value = value
  }

  public get enabled() {
    return this._enabled.value
  }

  public set enabled(value: boolean) {
    this._enabled.value = value
  }

  public get prompt() {
    return this._prompt.value
  }

  public set prompt(value: string) {
    this._prompt.value = value
  }

  public toPayload(): CreateMinionPayload {
    return {
      slug: this.slug.trim(),
      display_name: this.displayName.trim(),
      description: this.description.trim(),
      enabled: this.enabled,
      prompt: this.prompt.trim(),
    }
  }

  public toPayloadPatch(): MinionPatchPayload {
    const payload: MinionPatchPayload = {}

    if (this.canEditDefinition) {
      if (this._slug.isDirty) payload.slug = this.slug.trim()
      if (this._displayName.isDirty) payload.display_name = this.displayName.trim()
      if (this._description.isDirty) payload.description = this.description.trim()
      if (this._prompt.isDirty) payload.prompt = this.prompt.trim()
    }

    if (this.canToggleEnabled && this._enabled.isDirty) payload.enabled = this.enabled

    return payload
  }
}
