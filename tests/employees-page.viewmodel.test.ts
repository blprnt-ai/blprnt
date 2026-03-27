import assert from 'node:assert/strict'
import test from 'node:test'

import type { Employee } from '../src/bindings/Employee.ts'
import { employeesApi } from '../src/lib/api/employees.ts'
import { AppModel } from '../src/models/app.model.ts'
import { EmployeeViewmodel } from '../src/components/pages/employee/employee.viewmodel.ts'
import { EmployeesViewmodel } from '../src/components/pages/employees/employees.viewmodel.ts'

const owner: Employee = {
  id: 'owner-1',
  name: 'Owner',
  role: 'owner',
  kind: 'person',
  icon: 'briefcase',
  color: 'blue',
  title: 'Owner',
  status: 'idle',
  capabilities: [],
  permissions: null,
  reports_to: null,
  provider_config: null,
  runtime_config: null,
  chain_of_command: [],
}

const employeeFixture: Employee = {
  id: 'employee-1',
  name: 'Ada Lovelace',
  role: 'ceo',
  kind: 'agent',
  icon: 'brain',
  color: 'purple',
  title: 'Chief Executive Officer',
  status: 'running',
  capabilities: ['planning', 'strategy'],
  permissions: null,
  reports_to: owner.id,
  provider_config: {
    provider: 'claude_code',
    slug: 'ceo-agent',
  },
  runtime_config: {
    heartbeat_interval_sec: 3600,
    heartbeat_prompt: 'Review company goals.',
    max_concurrent_runs: 2,
    wake_on_demand: true,
  },
  chain_of_command: [owner],
}

const humanEmployeeFixture: Employee = {
  id: 'employee-2',
  name: 'Grace Hopper',
  role: 'manager',
  kind: 'person',
  icon: 'user',
  color: 'gray',
  title: 'Engineering Manager',
  status: 'idle',
  capabilities: ['planning'],
  permissions: null,
  reports_to: owner.id,
  provider_config: null,
  runtime_config: null,
  chain_of_command: [owner],
}

test('EmployeesViewmodel.init loads employees and syncs AppModel', async (t) => {
  const originalList = employeesApi.list
  const originalEmployees = AppModel.instance.employees

  t.after(() => {
    employeesApi.list = originalList
    AppModel.instance.setEmployees(originalEmployees)
  })

  employeesApi.list = async () => [owner, employeeFixture]

  const viewmodel = new EmployeesViewmodel()

  await viewmodel.init()

  assert.equal(viewmodel.employees.length, 2)
  assert.equal(viewmodel.employees[1]?.id, employeeFixture.id)
  assert.equal(AppModel.instance.employees.length, 2)
  assert.equal(AppModel.instance.resolveEmployeeName(employeeFixture.id), employeeFixture.name)
})

test('EmployeeViewmodel.init loads a single employee into editable state', async (t) => {
  const originalGet = employeesApi.get

  t.after(() => {
    employeesApi.get = originalGet
  })

  employeesApi.get = async () => employeeFixture

  const viewmodel = new EmployeeViewmodel(employeeFixture.id)

  await viewmodel.init()

  assert.equal(viewmodel.employee?.id, employeeFixture.id)
  assert.equal(viewmodel.employee?.name, employeeFixture.name)
  assert.equal(viewmodel.isEditing, false)
  assert.equal(viewmodel.showsAgentConfiguration, true)
})

test('EmployeeViewmodel hides agent configuration for human employees', async (t) => {
  const originalGet = employeesApi.get

  t.after(() => {
    employeesApi.get = originalGet
  })

  employeesApi.get = async () => humanEmployeeFixture

  const viewmodel = new EmployeeViewmodel(humanEmployeeFixture.id)

  await viewmodel.init()

  assert.equal(viewmodel.employee?.kind, 'person')
  assert.equal(viewmodel.showsAgentConfiguration, false)
})

test('EmployeeViewmodel.cancelEditing restores the original employee after unsaved changes', async (t) => {
  const originalGet = employeesApi.get

  t.after(() => {
    employeesApi.get = originalGet
  })

  employeesApi.get = async () => employeeFixture

  const viewmodel = new EmployeeViewmodel(employeeFixture.id)

  await viewmodel.init()
  viewmodel.startEditing()
  viewmodel.employee!.name = 'Temporary rename'
  viewmodel.employee!.title = 'Temporary title'

  viewmodel.cancelEditing()

  assert.equal(viewmodel.employee?.name, employeeFixture.name)
  assert.equal(viewmodel.employee?.title, employeeFixture.title)
  assert.equal(viewmodel.isEditing, false)
})

test('EmployeeViewmodel.save persists changes and upserts AppModel', async (t) => {
  const originalGet = employeesApi.get
  const originalUpdate = employeesApi.update
  const originalEmployees = AppModel.instance.employees

  t.after(() => {
    employeesApi.get = originalGet
    employeesApi.update = originalUpdate
    AppModel.instance.setEmployees(originalEmployees)
  })

  let payload: Parameters<typeof employeesApi.update>[1] | null = null

  employeesApi.get = async () => employeeFixture
  employeesApi.update = async (_id, data) => {
    payload = data

    return {
      ...employeeFixture,
      name: data.name ?? employeeFixture.name,
      title: data.title ?? employeeFixture.title,
    }
  }

  const viewmodel = new EmployeeViewmodel(employeeFixture.id)

  await viewmodel.init()
  viewmodel.startEditing()
  viewmodel.employee!.name = 'Grace Hopper'
  viewmodel.employee!.title = 'Chief Operating Officer'

  await viewmodel.save()

  assert.equal(payload?.name, 'Grace Hopper')
  assert.equal(payload?.title, 'Chief Operating Officer')
  assert.equal(viewmodel.employee?.name, 'Grace Hopper')
  assert.equal(viewmodel.employee?.title, 'Chief Operating Officer')
  assert.equal(viewmodel.isEditing, false)
  assert.equal(AppModel.instance.resolveEmployeeName(employeeFixture.id), 'Grace Hopper')
})
