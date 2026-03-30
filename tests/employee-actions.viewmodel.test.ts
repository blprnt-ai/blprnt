import assert from 'node:assert/strict'
import test from 'node:test'

import type { Employee } from '../src/bindings/Employee.ts'
import { EmployeeViewmodel } from '../src/components/pages/employee/employee.viewmodel.ts'
import { employeesApi } from '../src/lib/api/employees.ts'
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

const agentFixture: Employee = {
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
    heartbeat_interval_sec: 3600,
    heartbeat_prompt: 'Review company goals.',
    max_concurrent_runs: 2,
    wake_on_demand: true,
  },
  status: 'running',
  title: 'Chief Executive Officer',
}

test('EmployeeViewmodel.openAddIssue prefills the current agent as assignee', async (t) => {
  const originalGet = employeesApi.get

  t.after(() => {
    employeesApi.get = originalGet
  })

  employeesApi.get = async () => agentFixture

  const viewmodel = new EmployeeViewmodel(agentFixture.id)

  await viewmodel.init()
  viewmodel.openAddIssue()

  assert.equal(viewmodel.issueFormViewmodel.isOpen, true)
  assert.equal(viewmodel.issueFormViewmodel.issue.assignee, agentFixture.id)
})

test('EmployeeViewmodel.togglePaused pauses and resumes an agent', async (t) => {
  const originalGet = employeesApi.get
  const originalUpdate = employeesApi.update

  t.after(() => {
    employeesApi.get = originalGet
    employeesApi.update = originalUpdate
  })

  const requestedStatuses: string[] = []

  employeesApi.get = async () => agentFixture
  employeesApi.update = async (_id, data) => {
    requestedStatuses.push(data.status ?? 'missing')

    return {
      ...agentFixture,
      status: data.status ?? agentFixture.status,
    }
  }

  const viewmodel = new EmployeeViewmodel(agentFixture.id)

  await viewmodel.init()
  await viewmodel.togglePaused()
  await viewmodel.togglePaused()

  assert.deepEqual(requestedStatuses, ['paused', 'idle'])
  assert.equal(viewmodel.employee?.status, 'idle')
})

test('EmployeeViewmodel.terminate deletes the agent, updates AppModel, and triggers navigation callback', async (t) => {
  const originalGet = employeesApi.get
  const originalDelete = employeesApi.delete
  const originalEmployees = AppModel.instance.employees

  t.after(() => {
    employeesApi.get = originalGet
    employeesApi.delete = originalDelete
    AppModel.instance.setEmployees(originalEmployees)
  })

  let terminatedId: string | null = null
  let didNavigate = false

  employeesApi.get = async () => agentFixture
  employeesApi.delete = async (id) => {
    terminatedId = id
  }
  AppModel.instance.setEmployees([owner, agentFixture])

  const viewmodel = new EmployeeViewmodel(agentFixture.id, {
    onTerminated: async () => {
      didNavigate = true
    },
  })

  await viewmodel.init()
  const result = await viewmodel.terminate()

  assert.equal(result, true)
  assert.equal(terminatedId, agentFixture.id)
  assert.equal(didNavigate, true)
  assert.equal(AppModel.instance.resolveEmployeeName(agentFixture.id), null)
})
