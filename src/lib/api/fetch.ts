const API_BASE_URL = import.meta.env.VITE_API_URL

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

    return response.json()
  }
}

export const apiClient = new ApiClient()
