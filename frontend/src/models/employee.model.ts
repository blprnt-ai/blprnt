import { makeAutoObservable } from 'mobx'
import type { CreateEmployeePayload } from '@/bindings/CreateEmployeePayload'
import type { BootstrapOwnerPayload } from '@/bindings/BootstrapOwnerPayload'
import type { Employee } from '@/bindings/Employee'
import type { EmployeeKind } from '@/bindings/EmployeeKind'
import type { EmployeePatch } from '@/bindings/EmployeePatch'
import type { EmployeeProviderConfig } from '@/bindings/EmployeeProviderConfig'
import type { EmployeeRole } from '@/bindings/EmployeeRole'
import type { EmployeeRuntimeConfig } from '@/bindings/EmployeeRuntimeConfig'
import type { EmployeeSkillRef } from '@/bindings/EmployeeSkillRef'
import type { EmployeeStatus } from '@/bindings/EmployeeStatus'
import type { OwnerOnboardingPayload } from '@/bindings/OwnerOnboardingPayload'
import type { Provider } from '@/bindings/Provider'
import type { ReasoningEffort } from '@/bindings/ReasoningEffort'
import { type ColorVariant, colors } from '@/components/ui/colors'
import { employeeIcons } from '@/components/ui/employee-label'
import { isStructDirty, ModelField, type ModelStruct, structToPayload, structToPayloadPatch } from './model-field'

export class EmployeeModel {
  public id: string | null
  private _name: ModelField<string>
  private _kind: ModelField<EmployeeKind>
  private _role: ModelField<EmployeeRole>
  private _title: ModelField<string>
  private _status: ModelField<EmployeeStatus>
  private _icon: ModelField<string>
  private _color: ModelField<ColorVariant>
  private _email: ModelField<string>
  private _password: ModelField<string>
  private _capabilities: ModelField<string[]>
  private _reports_to: ModelField<string | null>
  private _provider_config: ModelStruct<EmployeeProviderConfig>
  private _runtime_config: ModelStruct<EmployeeRuntimeConfig>

  constructor(employee?: Employee) {
    this.id = employee?.id ?? null
    this._name = new ModelField(employee?.name ?? '')
    this._kind = new ModelField(employee?.kind ?? 'agent')
    this._role = new ModelField(employee?.role ?? 'manager')
    this._title = new ModelField(employee?.title ?? '')
    this._status = new ModelField(employee?.status ?? 'idle')
    this._icon = new ModelField(employee?.icon ?? 'bot')
    this._color = new ModelField((employee?.color as ColorVariant) ?? 'gray')
    this._email = new ModelField('')
    this._password = new ModelField('')
    this._capabilities = new ModelField(employee?.capabilities ?? [])
    this._reports_to = new ModelField(employee?.reports_to ?? null)
    this._provider_config = {
      provider: new ModelField(employee?.provider_config?.provider ?? 'claude_code'),
      slug: new ModelField(employee?.provider_config?.slug ?? ''),
    }
    this._runtime_config = {
      heartbeat_interval_sec: new ModelField(employee?.runtime_config?.heartbeat_interval_sec ?? 3600),
      heartbeat_prompt: new ModelField(employee?.runtime_config?.heartbeat_prompt ?? ''),
      max_concurrent_runs: new ModelField(employee?.runtime_config?.max_concurrent_runs ?? 1),
      reasoning_effort: new ModelField<ReasoningEffort | null>(employee?.runtime_config?.reasoning_effort ?? null),
      skill_stack: new ModelField<EmployeeSkillRef[] | null>(employee?.runtime_config?.skill_stack ?? null),
      wake_on_demand: new ModelField(employee?.runtime_config?.wake_on_demand ?? true),
    }

    makeAutoObservable(this)
  }

  public get isIdentityValid() {
    return (
      this.name.length > 0 &&
      this.icon.length > 0 &&
      this.color.length > 0 &&
      this.provider.length > 0 &&
      this.slug.length > 0
    )
  }

  public get isOwnerValid() {
    return this.name.length > 0 && this.icon.length > 0 && this.color.length > 0
  }

  public get isBootstrapOwnerValid() {
    return this.isOwnerValid && this.email.trim().length > 0 && this.password.trim().length >= 8
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
      this._email.isDirty ||
      this._password.isDirty ||
      this._capabilities.isDirty ||
      this._reports_to.isDirty ||
      isStructDirty(this._provider_config) ||
      isStructDirty(this._runtime_config)
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
    this._email.clearDirty()
    this._password.clearDirty()
    this._capabilities.clearDirty()
    this._reports_to.clearDirty()
    Object.values(this._provider_config).forEach((field) => field.clearDirty())
    Object.values(this._runtime_config).forEach((field) => field.clearDirty())
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

  public get email() {
    return this._email.value
  }

  public set email(email: string) {
    this._email.value = email
  }

  public get password() {
    return this._password.value
  }

  public set password(password: string) {
    this._password.value = password
  }

  public set capabilities(capabilities: string[]) {
    this._capabilities.value = capabilities
  }

  public get reports_to() {
    return this._reports_to.value
  }

  public set reports_to(reportsTo: string | null) {
    this._reports_to.value = reportsTo
  }

  public get provider() {
    return this._provider_config.provider.value
  }

  public set provider(provider: Provider) {
    this._provider_config.provider.value = provider
  }

  public setProvider(provider: Provider) {
    this._provider_config.provider.value = provider
  }

  public get slug() {
    return this._provider_config.slug.value
  }

  public set slug(slug: string) {
    this._provider_config.slug.value = slug
  }

  public get heartbeat_interval_sec() {
    return this._runtime_config.heartbeat_interval_sec.value
  }

  public set heartbeat_interval_sec(heartbeat_interval_sec: number) {
    this._runtime_config.heartbeat_interval_sec.value = heartbeat_interval_sec
  }

  public get heartbeat_prompt() {
    return this._runtime_config.heartbeat_prompt.value
  }

  public set heartbeat_prompt(heartbeat_prompt: string) {
    this._runtime_config.heartbeat_prompt.value = heartbeat_prompt
  }

  public get max_concurrent_runs() {
    return this._runtime_config.max_concurrent_runs.value
  }

  public set max_concurrent_runs(max_concurrent_runs: number) {
    this._runtime_config.max_concurrent_runs.value = max_concurrent_runs
  }

  public get wake_on_demand() {
    return this._runtime_config.wake_on_demand.value
  }

  public set wake_on_demand(wake_on_demand: boolean) {
    this._runtime_config.wake_on_demand.value = wake_on_demand
  }

  public get reasoning_effort() {
    return this._runtime_config.reasoning_effort.value
  }

  public set reasoning_effort(reasoning_effort: ReasoningEffort | null) {
    this._runtime_config.reasoning_effort.value = reasoning_effort
  }

  public get skill_stack() {
    return this._runtime_config.skill_stack.value ?? []
  }

  public set skill_stack(skill_stack: EmployeeSkillRef[]) {
    this._runtime_config.skill_stack.value = skill_stack.length > 0 ? skill_stack : null
  }

  public get selectedColor() {
    return colors.find((c) => c.color === this.color) ?? colors.find((c) => c.default)!
  }

  public get selectedIcon() {
    return employeeIcons.find((i) => i.value === this.icon) ?? employeeIcons.find((i) => i.default)!
  }

  public toOwnerOnboardingPayload(): OwnerOnboardingPayload {
    return {
      color: this._color.value,
      icon: this._icon.value,
      name: this._name.value,
    }
  }

  public toBootstrapOwnerPayload(): BootstrapOwnerPayload {
    return {
      color: this._color.value,
      email: this._email.value,
      icon: this._icon.value,
      name: this._name.value,
      password: this._password.value,
    }
  }

  public toPayload(): CreateEmployeePayload {
    return {
      capabilities: this._capabilities.value,
      color: this._color.value,
      icon: this._icon.value,
      kind: this._kind.value,
      name: this._name.value,
      provider_config: structToPayload(this._provider_config),
      role: this._role.value,
      runtime_config: structToPayload(this._runtime_config),
      heartbeat_md: null,
      soul_md: null,
      agents_md: null,
      tools_md: null,
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
      provider_config: structToPayloadPatch(this._provider_config),
      reports_to: this._reports_to.isDirty ? this._reports_to.value : (undefined as unknown as string | null),
      role: this._role.dirtyValue,
      runtime_config: structToPayloadPatch(this._runtime_config),
      status: this._status.dirtyValue,
      title: this._title.dirtyValue,
    }
  }
}
