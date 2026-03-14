export const planningPanelId = (projectId: string) => `planning-${projectId}`
export const projectPanelId = (projectId: string) => `project-${projectId}`
export const kanbanPanelId = (projectId: string) => `kanban-${projectId}`
export const previewPanelId = (projectId: string) => `preview-${projectId}`
export const planPanelId = (projectId: string, planId: string) => `plan-${projectId}-${planId}`

export const sessionPanelId = (projectId: string | undefined, sessionId: string) => `session-${projectId}-${sessionId}`
