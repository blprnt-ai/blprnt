import { makeAutoObservable } from 'mobx'

export class HeaderBreadcrumbModel {
  public labelsByRouteId = new Map<string, string>()

  public static instance = new HeaderBreadcrumbModel()

  private constructor() {
    makeAutoObservable(this)
  }

  public clearLabel(routeId: string) {
    this.labelsByRouteId.delete(routeId)
  }

  public getLabel(routeId: string) {
    return this.labelsByRouteId.get(routeId)
  }

  public setLabel(routeId: string, label: string) {
    this.labelsByRouteId.set(routeId, label)
  }
}
