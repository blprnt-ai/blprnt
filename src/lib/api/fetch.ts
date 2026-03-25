const API_BASE_URL = import.meta.env?.VITE_API_URL ?? 'http://localhost:9171/api/v1'

export class ApiError extends Error {
  public status: number

  constructor(message: string, status: number) {
    super(message)
    this.name = 'ApiError'
    this.status = status
  }
}

class ApiClient {
  private employeeId: string | null = null

  public setEmployeeId(employeeId: string): void {
    localStorage.setItem('employeeId', employeeId)
    this.employeeId = employeeId
  }

  public getEmployeeId(): string | null {
    if (this.employeeId) return this.employeeId

    const employeeId = localStorage.getItem('employeeId')
    if (!employeeId) return null

    this.employeeId = employeeId

    return employeeId
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
    const employeeId = this.getEmployeeId()
    const headers: Record<string, string> = {}
    if (employeeId) headers['x-blprnt-employee-id'] = employeeId
    if (options.body) headers['content-type'] = 'application/json'

    const response = await fetch(`${API_BASE_URL}${url}`, {
      ...options,
      headers: {
        ...options.headers,
        ...headers,
      },
    })
    const data = await this.parseBody(response)

    if (response.ok) {
      return data as T
    } else {
      throw new ApiError(this.errorMessage(data, response), response.status)
    }
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
