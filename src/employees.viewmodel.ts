import { makeAutoObservable, runInAction } from 'mobx'
import type { EmployeeStreamMessageDto } from '@/bindings/EmployeeStreamMessageDto'
import { connectEmployeesStream } from '@/lib/api/employees-stream'
import { AppModel } from '@/models/app.model'

export class EmployeesViewmodel {
  public isConnected = false
  public connectionError: string | null = null
  private socket: WebSocket | null = null
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null
  private reconnectDelayMs = 1000
  private employeeId: string | null = null

  constructor() {
    makeAutoObservable(this, {}, { autoBind: true })
  }

  public connect(employeeId: string) {
    this.employeeId = employeeId
    this.disconnect(false)
    this.openSocket()
  }

  public disconnect(resetEmployeeId = true) {
    if (this.reconnectTimer) clearTimeout(this.reconnectTimer)
    this.reconnectTimer = null
    if (this.socket) this.socket.close()
    this.socket = null

    runInAction(() => {
      this.isConnected = false
    })

    if (resetEmployeeId) this.employeeId = null
  }

  private openSocket() {
    if (!this.employeeId) return

    this.socket = connectEmployeesStream(this.employeeId, {
      onOpen: () => {
        runInAction(() => {
          this.isConnected = true
          this.connectionError = null
          this.reconnectDelayMs = 1000
        })
      },
      onMessage: (message) => {
        runInAction(() => {
          this.applyMessage(message)
        })
      },
      onError: () => {
        runInAction(() => {
          this.connectionError = 'Live employee updates disconnected.'
        })
      },
      onClose: () => {
        runInAction(() => {
          this.isConnected = false
        })
        this.scheduleReconnect()
      },
    })
  }

  private scheduleReconnect() {
    if (!this.employeeId) return
    if (this.reconnectTimer) clearTimeout(this.reconnectTimer)

    this.reconnectTimer = setTimeout(() => {
      this.openSocket()
    }, this.reconnectDelayMs)
    this.reconnectDelayMs = Math.min(this.reconnectDelayMs * 2, 10_000)
  }

  private applyMessage(message: EmployeeStreamMessageDto) {
    switch (message.type) {
      case 'snapshot':
        AppModel.instance.setEmployees(message.snapshot.employees)
        return
      case 'upsert':
        AppModel.instance.upsertEmployee(message.employee)
        return
      case 'delete':
        AppModel.instance.removeEmployee(message.employee_id)
        return
    }
  }
}
