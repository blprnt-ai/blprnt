import { toast } from 'sonner'
import type { CreateEmployeePayload } from '@/bindings/CreateEmployeePayload'
import type { Employee } from '@/bindings/Employee'
import type { EmployeePatch } from '@/bindings/EmployeePatch'
import type { OrgChart } from '@/bindings/OrgChart'
import type { OwnerOnboardingPayload } from '@/bindings/OwnerOnboardingPayload'
import { ApiError, apiClient } from './fetch'

class EmployeesApi {
  public async ownerOnboarding(data: OwnerOnboardingPayload): Promise<Employee> {
    return apiClient.post('/onboarding', {
      body: JSON.stringify(data),
    })
  }

  public async me(): Promise<Employee | null> {
    try {
      return await apiClient.get('/employees/me')
    } catch (error) {
      if (error instanceof ApiError && error.status === 400) {
        return null
      } else {
        toast.error('Failed to get owner. Please make sure the blprnt server is running.')
        return null
      }
    }
  }

  public async get(id: string): Promise<Employee> {
    return apiClient.get(`/employees/${id}`)
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
