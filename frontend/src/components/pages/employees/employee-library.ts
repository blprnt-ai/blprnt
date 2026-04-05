export const DEFAULT_EMPLOYEE_LIBRARY_BASE_URL = 'https://github.com/blprnt-ai/employees'

export interface EmployeeLibraryItem {
  id: string
  name: string
  description: string
  path: string
}

export interface EmployeeLibraryManifest {
  employees: EmployeeLibraryItem[]
  skills: EmployeeLibraryItem[]
}

export const resolveEmployeeLibraryImportBaseUrl = (baseUrl: string) => {
  const normalized = normalizeBaseUrl(baseUrl)
  if (!normalized) {
    throw new Error('Employee library base URL is required.')
  }

  if (!/^https?:\/\//i.test(normalized)) {
    return normalized
  }

  const url = new URL(normalized)
  if (url.hostname === 'github.com') {
    const [owner, repoWithSuffix] = url.pathname.split('/').filter(Boolean)
    if (!owner || !repoWithSuffix) {
      throw new Error('GitHub employee library base URL must include an owner and repository.')
    }

    return `https://github.com/${owner}/${repoWithSuffix.replace(/\.git$/, '')}`
  }

  const rawMatch =
    url.hostname === 'raw.githubusercontent.com' ? url.pathname.match(/^\/([^/]+)\/([^/]+)\/[^/]+(?:\/.*)?$/) : null
  if (rawMatch) {
    return `https://github.com/${rawMatch[1]}/${rawMatch[2].replace(/\.git$/, '')}`
  }

  if (url.pathname.endsWith('/manifest.json')) {
    url.pathname = url.pathname.replace(/\/manifest\.json$/, '')
  }

  return url.toString().replace(/\/+$/, '')
}

export const loadEmployeeLibraryManifest = async (baseUrl: string): Promise<EmployeeLibraryManifest> => {
  const manifestUrl = resolveManifestUrl(baseUrl)
  const response = await fetch(manifestUrl)

  if (!response.ok) {
    throw new Error(`Unable to load employee manifest from ${manifestUrl} (${response.status} ${response.statusText}).`)
  }

  let data: unknown
  try {
    data = await response.json()
  } catch {
    throw new Error(`Employee manifest at ${manifestUrl} is not valid JSON.`)
  }

  return validateManifest(data, manifestUrl)
}

const resolveManifestUrl = (baseUrl: string) => {
  const normalized = normalizeBaseUrl(baseUrl)
  if (!normalized) {
    throw new Error('Employee library base URL is required.')
  }

  const url = new URL(normalized)
  if (url.hostname === 'github.com') {
    const [owner, repoWithSuffix, mode, branch, ...rest] = url.pathname.split('/').filter(Boolean)
    if (!owner || !repoWithSuffix) {
      throw new Error('GitHub employee library base URL must include an owner and repository.')
    }

    const repo = repoWithSuffix.replace(/\.git$/, '')
    if (mode === 'blob' && branch && rest.join('/') === 'manifest.json') {
      return `https://raw.githubusercontent.com/${owner}/${repo}/${branch}/manifest.json`
    }

    const branchName = mode === 'tree' && branch ? branch : 'main'
    return `https://raw.githubusercontent.com/${owner}/${repo}/${branchName}/manifest.json`
  }

  if (url.pathname.endsWith('/manifest.json')) {
    return url.toString()
  }

  url.pathname = `${url.pathname.replace(/\/$/, '')}/manifest.json`
  return url.toString()
}

const normalizeBaseUrl = (baseUrl: string) => baseUrl.trim().replace(/\/+$/, '')

const validateManifest = (data: unknown, manifestUrl: string): EmployeeLibraryManifest => {
  if (!isRecord(data)) {
    throw new Error(`Employee manifest at ${manifestUrl} must be a JSON object.`)
  }

  const keys = Object.keys(data).sort()
  if (keys.length !== 2 || keys[0] !== 'employees' || keys[1] !== 'skills') {
    throw new Error(`Employee manifest at ${manifestUrl} must contain exactly two fields: employees and skills.`)
  }

  return {
    employees: validateItems(data.employees, 'employees', manifestUrl),
    skills: validateItems(data.skills, 'skills', manifestUrl),
  }
}

const validateItems = (value: unknown, fieldName: string, manifestUrl: string): EmployeeLibraryItem[] => {
  if (!Array.isArray(value)) {
    throw new Error(`Employee manifest field ${fieldName} at ${manifestUrl} must be an array.`)
  }

  return value.map((item, index) => validateItem(item, `${fieldName}[${index}]`, manifestUrl))
}

const validateItem = (value: unknown, fieldName: string, manifestUrl: string): EmployeeLibraryItem => {
  if (!isRecord(value)) {
    throw new Error(`Employee manifest entry ${fieldName} at ${manifestUrl} must be an object.`)
  }

  const keys = Object.keys(value).sort()
  if (keys.length !== 4 || keys.join(',') !== 'description,id,name,path') {
    throw new Error(
      `Employee manifest entry ${fieldName} at ${manifestUrl} must contain exactly id, name, description, and path.`,
    )
  }

  const item = {
    description: readString(value.description, `${fieldName}.description`, manifestUrl),
    id: readString(value.id, `${fieldName}.id`, manifestUrl),
    name: readString(value.name, `${fieldName}.name`, manifestUrl),
    path: readRelativePath(value.path, `${fieldName}.path`, manifestUrl),
  }

  return item
}

const readString = (value: unknown, fieldName: string, manifestUrl: string) => {
  if (typeof value !== 'string' || value.trim().length === 0) {
    throw new Error(`Employee manifest field ${fieldName} at ${manifestUrl} must be a non-empty string.`)
  }

  return value
}

const readRelativePath = (value: unknown, fieldName: string, manifestUrl: string) => {
  const path = readString(value, fieldName, manifestUrl)
  if (path.startsWith('/') || /^[a-z]+:\/\//i.test(path)) {
    throw new Error(`Employee manifest field ${fieldName} at ${manifestUrl} must be a relative path.`)
  }

  return path
}

const isRecord = (value: unknown): value is Record<string, unknown> => {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}
