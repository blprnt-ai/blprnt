import { makeObservable, observable } from 'mobx'
import { resolveApiBaseUrl } from './url'

const API_BASE_URL = resolveApiBaseUrl(import.meta.env?.VITE_API_URL)

export class ApiError extends Error {
  public status: number

  constructor(message: string, status: number) {
    super(message)
    this.name = 'ApiError'
    this.status = status
  }
}

class ApiClient {
  public employeeId: string | null = null
  private unauthorizedHandler: (() => void) | null = null

  constructor() {
    makeObservable(this, {
      employeeId: observable,
    })
  }

  public setEmployeeId(employeeId: string | null): void {
    this.employeeId = employeeId
  }

  public setUnauthorizedHandler(handler: (() => void) | null): void {
    this.unauthorizedHandler = handler
  }

  public get<T>(url: string, options: RequestInit = {}): Promise<T> {
    return this.fetch(url, { ...options, method: 'GET' })
  }

  public post<T>(url: string, options: RequestInit = {}): Promise<T> {
    return this.fetch(url, { ...options, method: 'POST' })
  }

  public patch<T>(url: string, options: RequestInit = {}): Promise<T> {
    return this.fetch(url, { ...options, method: 'PATCH' })
  }

  public delete<T>(url: string, options: RequestInit = {}): Promise<T> {
    return this.fetch(url, { ...options, method: 'DELETE' })
  }

  private async fetch<T>(url: string, options: RequestInit = {}): Promise<T> {
    const headers: Record<string, string> = {}
    if (this.employeeId) headers['x-blprnt-employee-id'] = this.employeeId
    if (options.body) headers['content-type'] = 'application/json'

    const response = await fetch(`${API_BASE_URL}${url}`, {
      ...options,
      credentials: 'include',
      headers: {
        ...options.headers,
        ...headers,
      },
    })
    const data = await this.parseBody(response)

    if (response.ok) {
      return data as T
    } else {
      const error = new ApiError(this.errorMessage(data, response), response.status)
      if (this.shouldHandleUnauthorized(url, error)) {
        this.unauthorizedHandler?.()
      }

      throw error
    }
  }

  private shouldHandleUnauthorized(url: string, error: ApiError) {
    if (url === '/auth/login' || url === '/auth/bootstrap-owner') return false

    return (
      error.status === 401 ||
      (error.status === 400 && error.message.includes('Employee header (x-blprnt-employee-id)'))
    )
  }

  private async parseBody(response: Response): Promise<unknown> {
    const text = await response.text()
    if (!text) {
      return undefined
    }

    if (response.headers.get('content-type')?.includes('application/json')) {
      return JSON.parse(text)
    }

    return text
  }

  private errorMessage(data: unknown, response: Response): string {
    if (typeof data === 'string' && data) {
      return data
    }

    if (typeof data === 'object' && data !== null && 'details' in data && typeof data.details === 'string') {
      return data.details
    }

    return response.statusText || `Request failed with status ${response.status}`
  }
}

export const apiClient = new ApiClient()
