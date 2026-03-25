import { makeAutoObservable } from 'mobx'
import type { CreateEmployeePayload } from '@/bindings/CreateEmployeePayload'
import type { Employee } from '@/bindings/Employee'
import type { EmployeeKind } from '@/bindings/EmployeeKind'
import type { EmployeePatch } from '@/bindings/EmployeePatch'
import type { EmployeeProviderConfig } from '@/bindings/EmployeeProviderConfig'
import type { EmployeeRole } from '@/bindings/EmployeeRole'
import type { EmployeeRuntimeConfig } from '@/bindings/EmployeeRuntimeConfig'
import type { EmployeeStatus } from '@/bindings/EmployeeStatus'
import type { OwnerOnboardingPayload } from '@/bindings/OwnerOnboardingPayload'
import { type ColorVariant, colors } from '@/components/ui/colors'
import { employeeIcons } from '@/components/ui/employee-label'
import { ModelField } from './model.field'

export class EmployeeModel {
  public id: string | null
  private _name: ModelField<string>
  private _kind: ModelField<EmployeeKind>
  private _role: ModelField<EmployeeRole>
  private _title: ModelField<string>
  private _status: ModelField<EmployeeStatus>
  private _icon: ModelField<string>
  private _color: ModelField<ColorVariant>
  private _capabilities: ModelField<string[]>
  private _provider_config: ModelField<EmployeeProviderConfig | null>
  private _runtime_config: ModelField<EmployeeRuntimeConfig | null>

  constructor(employee?: Employee) {
    this.id = employee?.id ?? null
    this._name = new ModelField(employee?.name ?? '')
    this._kind = new ModelField(employee?.kind ?? 'agent')
    this._role = new ModelField(employee?.role ?? 'manager')
    this._title = new ModelField(employee?.title ?? '')
    this._status = new ModelField(employee?.status ?? 'idle')
    this._icon = new ModelField(employee?.icon ?? 'bot')
    this._color = new ModelField((employee?.color as ColorVariant) ?? 'gray')
    this._capabilities = new ModelField(employee?.capabilities ?? [])
    this._provider_config = new ModelField(employee?.provider_config ?? null)
    this._runtime_config = new ModelField(employee?.runtime_config ?? null)

    makeAutoObservable(this)
  }

  public get isOwnerValid() {
    return this.name.length > 0 && this.icon.length > 0 && this.color.length > 0
  }

  public get isDirty() {
    return (
      this._name.isDirty ||
      this._kind.isDirty ||
      this._role.isDirty ||
      this._title.isDirty ||
      this._status.isDirty ||
      this._icon.isDirty ||
      this._color.isDirty ||
      this._capabilities.isDirty ||
      this._provider_config.isDirty ||
      this._runtime_config.isDirty
    )
  }

  public clearDirty() {
    this._name.clearDirty()
    this._kind.clearDirty()
    this._role.clearDirty()
    this._title.clearDirty()
    this._status.clearDirty()
    this._icon.clearDirty()
    this._color.clearDirty()
    this._capabilities.clearDirty()
    this._provider_config.clearDirty()
    this._runtime_config.clearDirty()
  }

  public get name() {
    return this._name.value
  }

  public set name(name: string) {
    this._name.value = name
  }

  public get kind() {
    return this._kind.value
  }

  public set kind(kind: EmployeeKind) {
    this._kind.value = kind
  }

  public get role() {
    return this._role.value
  }

  public set role(role: EmployeeRole) {
    this._role.value = role
  }

  public get title() {
    return this._title.value
  }

  public set title(title: string) {
    this._title.value = title
  }

  public get status() {
    return this._status.value
  }

  public set status(status: EmployeeStatus) {
    this._status.value = status
  }

  public get icon() {
    return this._icon.value
  }

  public set icon(icon: string) {
    this._icon.value = icon
  }

  public get color() {
    return this._color.value
  }

  public set color(color: ColorVariant) {
    this._color.value = color
  }

  public get capabilities() {
    return this._capabilities.value
  }

  public set capabilities(capabilities: string[]) {
    this._capabilities.value = capabilities
  }

  public get provider_config() {
    return this._provider_config.value
  }

  public set provider_config(provider_config: EmployeeProviderConfig | null) {
    this._provider_config.value = provider_config
  }

  public get runtime_config() {
    return this._runtime_config.value
  }

  public set runtime_config(runtime_config: EmployeeRuntimeConfig | null) {
    this._runtime_config.value = runtime_config
  }

  public get selectedColor() {
    return colors.find((c) => c.color === this.color)!
  }

  public get selectedIcon() {
    return employeeIcons.find((i) => i.value === this.icon)!
  }

  public toOwnerOnboardingPayload(): OwnerOnboardingPayload {
    return {
      color: this._color.value,
      icon: this._icon.value,
      name: this._name.value,
    }
  }

  public toPayload(): CreateEmployeePayload {
    return {
      capabilities: this._capabilities.value,
      color: this._color.value,
      icon: this._icon.value,
      kind: this._kind.value,
      name: this._name.value,
      provider_config: this._provider_config.value,
      role: this._role.value,
      runtime_config: this._runtime_config.value,
      title: this._title.value,
    }
  }

  public toPayloadPatch(): EmployeePatch {
    return {
      capabilities: this._capabilities.dirtyValue,
      color: this._color.dirtyValue,
      icon: this._icon.dirtyValue,
      last_run_at: null,
      name: this._name.dirtyValue,
      provider_config: this._provider_config.dirtyValue,
      reports_to: null,
      role: this._role.dirtyValue,
      runtime_config: this._runtime_config.dirtyValue,
      status: this._status.dirtyValue,
      title: this._title.dirtyValue,
    }
  }
}
