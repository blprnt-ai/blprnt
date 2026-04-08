import { makeAutoObservable } from 'mobx'
import type { CreateMcpServerPayload } from '@/bindings/CreateMcpServerPayload'
import type { McpServerDto } from '@/bindings/McpServerDto'
import type { McpServerPatchPayload } from '@/bindings/McpServerPatchPayload'

export class McpServerFormModel {
  public id: string | null = null
  public displayName = ''
  public description = ''
  public transport = 'streamable_http'
  public endpointUrl = ''
  public enabled = true

  constructor(server?: McpServerDto | null) {
    makeAutoObservable(this)
    if (server) this.setFromDto(server)
  }

  public get isNew() {
    return this.id === null
  }

  public get isValid() {
    return (
      this.displayName.trim().length > 0 && this.description.trim().length > 0 && this.endpointUrl.trim().length > 0
    )
  }

  public reset() {
    this.id = null
    this.displayName = ''
    this.description = ''
    this.transport = 'streamable_http'
    this.endpointUrl = ''
    this.enabled = true
  }

  public setFromDto(server: McpServerDto) {
    this.id = server.id
    this.displayName = server.display_name
    this.description = server.description
    this.transport = server.transport
    this.endpointUrl = server.endpoint_url
    this.enabled = server.enabled
  }

  public toCreatePayload(): CreateMcpServerPayload {
    return {
      description: this.description.trim(),
      display_name: this.displayName.trim(),
      enabled: this.enabled,
      endpoint_url: this.endpointUrl.trim(),
      transport: this.transport.trim(),
    }
  }

  public toPatchPayload(): McpServerPatchPayload {
    return {
      description: this.description.trim(),
      display_name: this.displayName.trim(),
      enabled: this.enabled,
      endpoint_url: this.endpointUrl.trim(),
      transport: this.transport.trim(),
    }
  }
}
