export const projectNodeId = (projectId: string) => `project-${projectId}`
export const planningNodeId = (projectId: string) => `planning-${projectId}`
export const sessionsNodeId = (projectId: string) => `sessions-${projectId}`
export const sessionNodeId = (projectId: string | undefined, sessionId: string) => `session-${projectId}-${sessionId}`
