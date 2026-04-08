import assert from 'node:assert/strict'
import { test } from 'vitest'

import type { Employee } from '../src/bindings/Employee.ts'
import type { EmployeeLifeFileResult } from '../src/bindings/EmployeeLifeFileResult.ts'
import type { EmployeeLifeTreeResult } from '../src/bindings/EmployeeLifeTreeResult.ts'
import { EmployeeViewmodel } from '../src/components/pages/employee/employee.viewmodel.ts'
import { employeesApi } from '../src/lib/api/employees.ts'
import { skillsApi } from '../src/lib/api/skills.ts'

const owner: Employee = {
  capabilities: [],
  chain_of_command: [],
  color: 'blue',
  created_at: '2026-04-01T00:00:00Z',
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
  capabilities: ['planning'],
  chain_of_command: [owner],
  color: 'purple',
  created_at: '2026-04-01T00:00:00Z',
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
    timer_wakeups_enabled: true,
    wake_on_demand: true,
    reasoning_effort: null,
    skill_stack: null,
  },
  status: 'running',
  title: 'Chief Executive Officer',
}

const lifeTreeFixture: EmployeeLifeTreeResult = {
  root_path: '$AGENT_HOME',
  nodes: [
    { editable: true, kind: 'home_doc', name: 'HEARTBEAT.md', path: 'HEARTBEAT.md', type: 'file' },
    { editable: true, kind: 'home_doc', name: 'SOUL.md', path: 'SOUL.md', type: 'file' },
    {
      children: [
        { editable: false, kind: 'memory', name: '2026-04-01.md', path: 'memory/2026-04-01.md', type: 'file' },
      ],
      name: 'memory',
      path: 'memory',
      type: 'directory',
    },
  ],
}

test('EmployeeViewmodel.init loads life tree and selects HEARTBEAT.md first', async (t) => {
  const originalGet = employeesApi.get
  const originalLife = employeesApi.life
  const originalReadLifeFile = employeesApi.readLifeFile
  const originalListSkills = skillsApi.list

  t.onTestFinished(() => {
    employeesApi.get = originalGet
    employeesApi.life = originalLife
    employeesApi.readLifeFile = originalReadLifeFile
    skillsApi.list = originalListSkills
  })

  employeesApi.get = async () => employeeFixture
  employeesApi.life = async () => lifeTreeFixture
  employeesApi.readLifeFile = async (id, path) => {
    assert.equal(id, employeeFixture.id)
    assert.equal(path, 'HEARTBEAT.md')

    return {
      content: '# Focus\n',
      editable: true,
      kind: 'home_doc',
      path,
    }
  }
  skillsApi.list = async () => []

  const viewmodel = new EmployeeViewmodel(employeeFixture.id)

  await viewmodel.init()

  assert.equal(viewmodel.selectedLifePath, 'HEARTBEAT.md')
  assert.equal(viewmodel.lifeFile?.path, 'HEARTBEAT.md')
  assert.equal(viewmodel.lifeDraftContent, '# Focus\n')
  assert.equal(viewmodel.canEditSelectedLifeFile, true)
})

test('EmployeeViewmodel.saveLifeFile persists editable home docs without touching memory files', async (t) => {
  const originalGet = employeesApi.get
  const originalLife = employeesApi.life
  const originalReadLifeFile = employeesApi.readLifeFile
  const originalUpdateLifeFile = employeesApi.updateLifeFile
  const originalListSkills = skillsApi.list

  t.onTestFinished(() => {
    employeesApi.get = originalGet
    employeesApi.life = originalLife
    employeesApi.readLifeFile = originalReadLifeFile
    employeesApi.updateLifeFile = originalUpdateLifeFile
    skillsApi.list = originalListSkills
  })

  let updatePayload: { path: string; content: string } | null = null

  employeesApi.get = async () => employeeFixture
  employeesApi.life = async () => lifeTreeFixture
  employeesApi.readLifeFile = async (_id, path): Promise<EmployeeLifeFileResult> => ({
    content: path === 'HEARTBEAT.md' ? '# Focus\n' : '# Notes\n',
    editable: path === 'HEARTBEAT.md',
    kind: path === 'HEARTBEAT.md' ? 'home_doc' : 'memory',
    path,
  })
  employeesApi.updateLifeFile = async (_id, data) => {
    updatePayload = data

    return {
      content: data.content,
      editable: true,
      kind: 'home_doc',
      path: data.path,
    }
  }
  skillsApi.list = async () => []

  const viewmodel = new EmployeeViewmodel(employeeFixture.id)

  await viewmodel.init()
  viewmodel.setLifeDraftContent('# New Focus\n')
  await viewmodel.saveLifeFile()

  assert.deepEqual(updatePayload, {
    content: '# New Focus\n',
    path: 'HEARTBEAT.md',
  })
  assert.equal(viewmodel.lifeFile?.content, '# New Focus\n')

  await viewmodel.selectLifePath('memory/2026-04-01.md')
  updatePayload = null
  await viewmodel.saveLifeFile()

  assert.equal(updatePayload, null)
  assert.equal(viewmodel.canEditSelectedLifeFile, false)
})
