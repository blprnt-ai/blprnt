import {
  commands,
  type DeleteQueuedPromptOutcome,
  type DeleteQueuedPromptRequest,
  type SessionCreateParams,
  type SessionPatchV2,
} from '@/bindings'
import { EventType, globalEventBus } from '@/lib/events'

class TauriSessionApi {
  public async list(projectId: string) {
    const result = await commands.sessionList(projectId)
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async create(params: SessionCreateParams) {
    const result = await commands.sessionCreate(params)
    if (result.status === 'error') throw result.error

    globalEventBus.emit(EventType.Internal, {
      event: {
        sessionId: result.data.id,
        type: 'session_added',
      },
    })

    return result.data
  }

  public async update(sessionId: string, patch: SessionPatchV2) {
    const result = await commands.sessionUpdate(sessionId, patch)
    if (result.status === 'error') throw result.error

    globalEventBus.emit(EventType.Internal, {
      event: {
        sessionId: sessionId,
        type: 'session_updated',
      },
    })

    return result.data
  }

  public async listMessages(sessionId: string) {
    const result = await commands.sessionHistory(sessionId)
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async deleteMessage(messageId: string) {
    const result = await commands.deleteMessage(messageId)
    if (result.status === 'error') throw result.error
  }

  public async start(sessionId: string) {
    const result = await commands.sessionStart(sessionId)
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async get(sessionId: string) {
    const result = await commands.sessionGet(sessionId)
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async stop(sessionId: string) {
    const result = await commands.sessionStop(sessionId)
    if (result.status === 'error') throw result.error
  }

  public async delete(sessionId: string) {
    const result = await commands.sessionDelete(sessionId)
    if (result.status === 'error') throw result.error

    globalEventBus.emit(EventType.Internal, {
      event: {
        sessionId: sessionId,
        type: 'session_removed',
      },
    })
  }

  public async assignPlanToSession(sessionId: string, planId: string) {
    const result = await commands.assignPlanToSession(sessionId, planId)
    if (result.status === 'error') throw result.error
  }

  public async unassignPlanFromSession(sessionId: string, planId: string) {
    const result = await commands.unassignPlanFromSession(sessionId, planId)
    if (result.status === 'error') throw result.error
  }

  public async sendPrompt(sessionId: string, prompt: string, imageUrls: string[] | null) {
    const result = await commands.sendPrompt(sessionId, prompt, imageUrls)
    if (result.status === 'error') throw result.error
  }

  public async deleteQueuedPrompt(request: DeleteQueuedPromptRequest): Promise<DeleteQueuedPromptOutcome> {
    const result = await commands.deleteQueuedPrompt(request)
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async startPlanBuild(sessionId: string, planId: string) {
    const result = await commands.startPlan(sessionId, planId)
    if (result.status === 'error') throw result.error
  }

  public async continuePlanBuild(sessionId: string, planId: string) {
    const result = await commands.continuePlan(sessionId, planId)
    if (result.status === 'error') throw result.error
  }

  public async completePlan(sessionId: string, planId: string) {
    const result = await commands.completePlan(sessionId, planId)
    if (result.status === 'error') throw result.error

    globalEventBus.emit(EventType.Internal, {
      event: {
        planId: planId,
        sessionId: sessionId,
        type: 'plan_completed',
      },
    })
  }

  public async cancelPlan(sessionId: string, planId: string) {
    const result = await commands.cancelPlan(sessionId, planId)
    if (result.status === 'error') throw result.error
  }

  public async deletePlan(sessionId: string, planId: string) {
    const result = await commands.deletePlan(sessionId, planId)
    if (result.status === 'error') throw result.error
  }

  public async rewindTo(sessionId: string, historyId: string) {
    const result = await commands.rewindTo(sessionId, historyId)
    if (result.status === 'error') throw result.error
  }

  public async sendInterrupt(sessionId: string) {
    const result = await commands.sendInterrupt(sessionId)
    if (result.status === 'error') throw result.error
  }

  public async submitAnswer(sessionId: string, questionId: string, answer: string) {
    const result = await commands.answerQuestion(sessionId, questionId, answer)
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async listSkills() {
    const result = await commands.listSkills()
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async getTerminalSnapshot(sessionId: string, terminalId: string) {
    const result = await commands.getTerminalSnapshot(sessionId, terminalId)
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async closeTerminal(sessionId: string, terminalId: string) {
    const result = await commands.closeTerminal(sessionId, terminalId)
    if (result.status === 'error') throw result.error
  }
}

export const tauriSessionApi = new TauriSessionApi()
