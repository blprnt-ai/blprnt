import type { CreateEmployeePayload } from '@/bindings/CreateEmployeePayload'
import type { Employee } from '@/bindings/Employee'
import type { EmployeeLifeFilePatchPayload } from '@/bindings/EmployeeLifeFilePatchPayload'
import type { EmployeeLifeFileResult } from '@/bindings/EmployeeLifeFileResult'
import type { EmployeeLifeTreeResult } from '@/bindings/EmployeeLifeTreeResult'
import type { EmployeePatch } from '@/bindings/EmployeePatch'
import type { ImportEmployeePayload } from '@/bindings/ImportEmployeePayload'
import type { OrgChart } from '@/bindings/OrgChart'
import type { OwnerOnboardingPayload } from '@/bindings/OwnerOnboardingPayload'
import { apiClient } from './fetch'

class EmployeesApi {
  public async ownerOnboarding(data: OwnerOnboardingPayload): Promise<Employee> {
    return apiClient.post('/onboarding', {
      body: JSON.stringify(data),
    })
  }

  public async getOwner(): Promise<Employee | null> {
    return apiClient.get('/owner')
  }

  public async me(): Promise<Employee | null> {
    return apiClient.get('/employees/me')
  }

  public async get(id: string): Promise<Employee> {
    return apiClient.get(`/employees/${id}`)
  }

  public async life(id: string): Promise<EmployeeLifeTreeResult> {
    return apiClient.get(`/employees/${id}/life`)
  }

  public async readLifeFile(id: string, path: string): Promise<EmployeeLifeFileResult> {
    return apiClient.get(`/employees/${id}/life/file?path=${encodeURIComponent(path)}`)
  }

  public async updateLifeFile(id: string, data: EmployeeLifeFilePatchPayload): Promise<EmployeeLifeFileResult> {
    return apiClient.patch(`/employees/${id}/life/file`, {
      body: JSON.stringify(data),
    })
  }

  public async list(): Promise<Employee[]> {
    return apiClient.get('/employees')
  }

  public async orgChart(): Promise<OrgChart[]> {
    return apiClient.get('/employees/org-chart')
  }

  public async create(data: CreateEmployeePayload): Promise<Employee> {
    return apiClient.post('/employees', {
      body: JSON.stringify(data),
    })
  }

  public async import(data: ImportEmployeePayload): Promise<Employee> {
    return apiClient.post('/employees/import', {
      body: JSON.stringify(data),
    })
  }

  public async update(id: string, data: EmployeePatch): Promise<Employee> {
    return apiClient.patch(`/employees/${id}`, {
      body: JSON.stringify(data),
    })
  }

  public async delete(id: string): Promise<void> {
    return apiClient.delete(`/employees/${id}`)
  }
}

export const employeesApi = new EmployeesApi()
