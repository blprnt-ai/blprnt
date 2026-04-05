import { beforeEach, describe, expect, it } from 'vitest'
import type { Employee } from '@/bindings/Employee'
import type { EmployeeStreamMessageDto } from '@/bindings/EmployeeStreamMessageDto'
import { AppModel } from '@/models/app.model'
import { EmployeesViewmodel } from '@/employees.viewmodel'

const buildEmployee = (overrides: Partial<Employee> = {}): Employee => ({
  id: '00000000-0000-0000-0000-000000000001',
  name: 'Ada',
  role: 'staff',
  kind: 'agent',
  icon: 'bot',
  color: 'blue',
  title: 'Engineer',
  status: 'active',
  capabilities: [],
  permissions: null,
  reports_to: null,
  provider_config: null,
  runtime_config: null,
  chain_of_command: [],
  ...overrides,
})

describe('EmployeesViewmodel', () => {
  beforeEach(() => {
    AppModel.instance.employees = []
  })

  it('applies snapshot, upsert, and delete messages to the global employee list', () => {
    const viewmodel = new EmployeesViewmodel()

    ;(viewmodel as { applyMessage: (message: EmployeeStreamMessageDto) => void }).applyMessage({
      type: 'snapshot',
      snapshot: {
        employees: [buildEmployee()],
      },
    })

    expect(AppModel.instance.employees.map((employee) => employee.name)).toEqual(['Ada'])

    ;(viewmodel as { applyMessage: (message: EmployeeStreamMessageDto) => void }).applyMessage({
      type: 'upsert',
      employee: buildEmployee({
        id: '00000000-0000-0000-0000-000000000002',
        name: 'Bea',
      }),
    })

    expect(AppModel.instance.employees.map((employee) => employee.name)).toEqual(['Ada', 'Bea'])

    ;(viewmodel as { applyMessage: (message: EmployeeStreamMessageDto) => void }).applyMessage({
      type: 'delete',
      employee_id: '00000000-0000-0000-0000-000000000001',
    })

    expect(AppModel.instance.employees.map((employee) => employee.name)).toEqual(['Bea'])
  })
})
