import { makeAutoObservable } from 'mobx'
import type { Employee } from '@/bindings/Employee'
import { employeesApi } from '@/lib/api/employees'
import { EmployeeModel } from '@/models/employee.model'

export class EmployeeFormViewmodel {
  public employee: EmployeeModel = new EmployeeModel()

  constructor() {
    makeAutoObservable(this)
  }

  public init = async (employeeId?: string) => {
    if (!employeeId) return

    const employee = await employeesApi.get(employeeId)
    this.setEmployee(employee)
  }

  private setEmployee = (employee: Employee) => {
    this.employee = new EmployeeModel(employee)
  }

  public save = async () => {
    if (!this.employee.isDirty) return

    if (!this.employee.id) await this.createEmployee()
    else await this.updateEmployee()
  }

  private createEmployee = async () => {
    const employee = await employeesApi.create(this.employee.toPayload())
    this.setEmployee(employee)
  }

  private updateEmployee = async () => {
    const employee = await employeesApi.update(this.employee.id!, this.employee.toPayloadPatch())
    this.setEmployee(employee)
  }
}
