import { makeAutoObservable } from 'mobx'

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

  public clearDirty() {
    this._isDirty = false
  }

  public reset() {
    this._value = this._initialValue
    this._isDirty = false
  }
}
