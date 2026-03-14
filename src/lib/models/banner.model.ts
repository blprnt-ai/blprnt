import { makeAutoObservable } from 'mobx'

export class BannerModel {
  public content: string = ''
  public type: 'warning' | 'info' = 'info'
  public showBanner: boolean = false
  public action: () => void = () => {}

  constructor() {
    makeAutoObservable(this, {}, { autoBind: true })
  }

  public setContent(content: string) {
    this.content = content
  }

  public setShowBanner(showBanner: boolean) {
    this.showBanner = showBanner
  }

  public setAction(action: () => void) {
    this.action = action
  }

  public setType(type: 'warning' | 'info') {
    this.type = type
  }

  public dismiss(e?: React.MouseEvent<HTMLButtonElement>) {
    e?.stopPropagation()
    e?.preventDefault()

    this.type = 'info'
    this.content = ''
    this.showBanner = false
    this.action = () => {}
  }

  public clickAction() {
    this.action()
    this.dismiss()
  }
}
