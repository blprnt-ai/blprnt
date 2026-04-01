import { makeAutoObservable } from 'mobx'

export type ModelStruct<T> = {
  [K in keyof T]: ModelField<T[K]>
}

export const isStructDirty = <T>(struct: ModelStruct<T>) =>
  (Object.values(struct) as ModelField<T[keyof T]>[]).some((field) => field.isDirty)

export const structToPayload = <T>(struct: ModelStruct<T>) =>
  Object.fromEntries(
    (Object.entries(struct) as [keyof T, ModelField<T[keyof T]>][]).map(([key, field]) => [key, field.value]),
  ) as T

export const structToPayloadPatch = <T>(struct: ModelStruct<T>) =>
  isStructDirty(struct) ? structToPayload(struct) : null

export const structReset = <T>(struct: ModelStruct<T>) =>
  (Object.values(struct) as ModelField<T[keyof T]>[]).forEach((field) => field.reset())

export class ModelField<T> {
  private _initialValue: T
  private _value: T
  private _isDirty: boolean = false

  constructor(value: T) {
    this._initialValue = value
    this._value = value

    makeAutoObservable(this)
  }

  public get value() {
    return this._value
  }

  public get dirtyValue() {
    return this._isDirty ? this._value : null
  }

  public set value(value: T) {
    this._value = value
    this._isDirty = true
  }

  public get isDirty() {
    return this._isDirty
  }

  public get isSame() {
    return this._value === this._initialValue
  }

  public clearDirty() {
    this._isDirty = false
  }

  public reset() {
    this._value = this._initialValue
    this._isDirty = false
  }
}
