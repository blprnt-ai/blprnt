import assert from 'node:assert/strict'
import { test } from 'vitest'

import type { Employee } from '../src/bindings/Employee.ts'
import type { ProviderDto } from '../src/bindings/ProviderDto.ts'
import { EmployeeViewmodel } from '../src/components/pages/employee/employee.viewmodel.ts'
import { EmployeesViewmodel } from '../src/components/pages/employees/employees.viewmodel.ts'
import { employeesApi } from '../src/lib/api/employees.ts'
import { providersApi } from '../src/lib/api/providers.ts'
import { AppModel } from '../src/models/app.model.ts'

const owner: Employee = {
  capabilities: [],
  chain_of_command: [],
  color: 'blue',
  icon: 'briefcase',
  id: 'owner-1',
  kind: 'person',
  name: 'Owner',
  permissions: null,
  provider_config: null,
  reports_to: null,
  role: 'owner',
  runtime_config: null,
  status: 'idle',
  title: 'Owner',
}

const employeeFixture: Employee = {
  capabilities: ['planning', 'strategy'],
  chain_of_command: [owner],
  color: 'purple',
  icon: 'brain',
  id: 'employee-1',
  kind: 'agent',
  name: 'Ada Lovelace',
  permissions: null,
  provider_config: {
    provider: 'claude_code',
    slug: 'ceo-agent',
  },
  reports_to: owner.id,
  role: 'ceo',
  runtime_config: {
    dreams_enabled: false,
    heartbeat_interval_sec: 3600,
    heartbeat_prompt: 'Review company goals.',
    max_concurrent_runs: 2,
    prevent_empty_runs: false,
    timer_wakeups_enabled: true,
    wake_on_demand: true,
    reasoning_effort: null,
    skill_stack: null,
  },
  status: 'running',
  title: 'Chief Executive Officer',
}

const humanEmployeeFixture: Employee = {
  capabilities: ['planning'],
  chain_of_command: [owner],
  color: 'gray',
  icon: 'user',
  id: 'employee-2',
  kind: 'person',
  name: 'Grace Hopper',
  permissions: null,
  provider_config: null,
  reports_to: owner.id,
  role: 'manager',
  runtime_config: null,
  status: 'idle',
  title: 'Engineering Manager',
}

const openAiProviderFixture: ProviderDto = {
  base_url: 'https://api.openai.com/v1',
  created_at: '2026-04-01T12:00:00.000Z',
  id: 'provider-openai',
  provider: 'openai',
  updated_at: '2026-04-01T12:00:00.000Z',
}

test('EmployeesViewmodel.init loads employees and syncs AppModel', async (t) => {
  const originalList = employeesApi.list
  const originalEmployees = AppModel.instance.employees

  t.onTestFinished(() => {
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

  t.onTestFinished(() => {
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

  t.onTestFinished(() => {
    employeesApi.get = originalGet
  })

  employeesApi.get = async () => humanEmployeeFixture

  const viewmodel = new EmployeeViewmodel(humanEmployeeFixture.id)

  await viewmodel.init()

  assert.equal(viewmodel.employee?.kind, 'person')
  assert.equal(viewmodel.showsAgentConfiguration, false)
})

test('EmployeeViewmodel disables unconfigured runtime providers while keeping the current provider selectable', async (t) => {
  const originalGet = employeesApi.get
  const originalListProviders = providersApi.list

  t.onTestFinished(() => {
    employeesApi.get = originalGet
    providersApi.list = originalListProviders
  })

  employeesApi.get = async () => employeeFixture
  providersApi.list = async () => [openAiProviderFixture]

  const viewmodel = new EmployeeViewmodel(employeeFixture.id)

  await viewmodel.init()

  assert.equal(viewmodel.runtimeProviderOptions.find((option) => option.value === 'claude_code')?.disabled, false)
  assert.equal(viewmodel.runtimeProviderOptions.find((option) => option.value === 'openai')?.disabled, false)
  assert.equal(viewmodel.runtimeProviderOptions.find((option) => option.value === 'anthropic')?.disabled, true)
  assert.equal(viewmodel.runtimeProviderOptions.find((option) => option.value === 'codex')?.disabled, true)
})

test('EmployeeViewmodel.cancelEditing restores the original employee after unsaved changes', async (t) => {
  const originalGet = employeesApi.get

  t.onTestFinished(() => {
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

  t.onTestFinished(() => {
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

test('EmployeeViewmodel.save persists timer wakeup changes and treats legacy missing values safely', async (t) => {
  const originalGet = employeesApi.get
  const originalUpdate = employeesApi.update

  t.onTestFinished(() => {
    employeesApi.get = originalGet
    employeesApi.update = originalUpdate
  })

  let payload: Parameters<typeof employeesApi.update>[1] | null = null

  employeesApi.get = async () => ({
    ...employeeFixture,
    runtime_config: {
      ...employeeFixture.runtime_config!,
      timer_wakeups_enabled: null,
    },
  })
  employeesApi.update = async (_id, data) => {
    payload = data

    return {
      ...employeeFixture,
      runtime_config: {
        ...employeeFixture.runtime_config!,
        timer_wakeups_enabled: data.runtime_config?.timer_wakeups_enabled ?? true,
      },
    }
  }

  const viewmodel = new EmployeeViewmodel(employeeFixture.id)

  await viewmodel.init()

  assert.equal(viewmodel.employee?.timer_wakeups_enabled, true)

  viewmodel.employee!.timer_wakeups_enabled = false
  await viewmodel.save()

  assert.equal(payload?.runtime_config?.timer_wakeups_enabled, false)
  assert.equal(viewmodel.employee?.timer_wakeups_enabled, false)
})

test('EmployeeViewmodel.save persists dreaming changes and treats legacy missing values as disabled', async (t) => {
  const originalGet = employeesApi.get
  const originalUpdate = employeesApi.update

  t.onTestFinished(() => {
    employeesApi.get = originalGet
    employeesApi.update = originalUpdate
  })

  let payload: Parameters<typeof employeesApi.update>[1] | null = null

  employeesApi.get = async () => ({
    ...employeeFixture,
    runtime_config: {
      ...employeeFixture.runtime_config!,
      dreams_enabled: null,
    },
  })
  employeesApi.update = async (_id, data) => {
    payload = data

    return {
      ...employeeFixture,
      runtime_config: {
        ...employeeFixture.runtime_config!,
        dreams_enabled: data.runtime_config?.dreams_enabled ?? false,
      },
    }
  }

  const viewmodel = new EmployeeViewmodel(employeeFixture.id)

  await viewmodel.init()

  assert.equal(viewmodel.employee?.dreams_enabled, false)

  viewmodel.employee!.dreams_enabled = true
  await viewmodel.save()

  assert.equal(payload?.runtime_config?.dreams_enabled, true)
  assert.equal(viewmodel.employee?.dreams_enabled, true)
})

test('EmployeeViewmodel.save persists prevent empty runs changes and treats legacy missing values as disabled', async (t) => {
  const originalGet = employeesApi.get
  const originalUpdate = employeesApi.update

  t.onTestFinished(() => {
    employeesApi.get = originalGet
    employeesApi.update = originalUpdate
  })

  let payload: Parameters<typeof employeesApi.update>[1] | null = null

  employeesApi.get = async () => ({
    ...employeeFixture,
    runtime_config: {
      ...employeeFixture.runtime_config!,
      prevent_empty_runs: null,
    },
  })
  employeesApi.update = async (_id, data) => {
    payload = data

    return {
      ...employeeFixture,
      runtime_config: {
        ...employeeFixture.runtime_config!,
        prevent_empty_runs: data.runtime_config?.prevent_empty_runs ?? false,
      },
    }
  }

  const viewmodel = new EmployeeViewmodel(employeeFixture.id)

  await viewmodel.init()

  assert.equal(viewmodel.employee?.prevent_empty_runs, false)

  viewmodel.employee!.prevent_empty_runs = true
  await viewmodel.save()

  assert.equal(payload?.runtime_config?.prevent_empty_runs, true)
  assert.equal(viewmodel.employee?.prevent_empty_runs, true)
})
