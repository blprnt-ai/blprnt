import type { AgentKind } from '@/bindings'

export const agentKindLabel = (agentKind: AgentKind) => {
  if (agentKind === 'crew') return 'Crew'
  if (agentKind === 'planner') return 'Planner'
  if (agentKind === 'executor') return 'Executor'
  if (agentKind === 'verifier') return 'Verifier'
  if (agentKind === 'designer') return 'Designer'

  return agentKind
}
