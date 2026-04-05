import type { AuthStatusDto } from '@/bindings/AuthStatusDto'
import type { BootstrapOwnerPayload } from '@/bindings/BootstrapOwnerPayload'
import type { Employee } from '@/bindings/Employee'
import type { LoginPayload } from '@/bindings/LoginPayload'
import { apiClient } from './fetch'

class AuthApi {
  public async status(): Promise<AuthStatusDto> {
    return apiClient.get('/auth/status')
  }

  public async bootstrapOwner(data: BootstrapOwnerPayload): Promise<Employee> {
    return apiClient.post('/auth/bootstrap-owner', {
      body: JSON.stringify(data),
    })
  }

  public async login(data: LoginPayload): Promise<Employee> {
    return apiClient.post('/auth/login', {
      body: JSON.stringify(data),
    })
  }

  public async me(): Promise<Employee> {
    return apiClient.get('/auth/me')
  }

  public async logout(): Promise<void> {
    return apiClient.post('/auth/logout')
  }
}

export const authApi = new AuthApi()
