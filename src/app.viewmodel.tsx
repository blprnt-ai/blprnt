import { makeAutoObservable } from 'mobx'
import { createContext, useContext } from 'react'
import { employeesApi } from './lib/api/employees'
import { AppModel } from './models/app.model'

export class AppViewmodel {
  constructor() {
    makeAutoObservable(this)
  }

  public init = async () => {
    const owner = await employeesApi.getOwner()
    if (owner) AppModel.instance.setOwner(owner)
  }

  public get isOnboarded() {
    return AppModel.instance.isOnboarded
  }

  public setIsOnboarded(isOnboarded: boolean) {
    AppModel.instance.setIsOnboarded(isOnboarded)
  }
}

export const AppViewmodelContext = createContext<AppViewmodel | null>(null)
export const useAppViewmodel = () => {
  const appViewmodel = useContext(AppViewmodelContext)
  if (!appViewmodel) throw new Error('AppViewmodel not found')

  return appViewmodel
}
