import { makeAutoObservable } from 'mobx'
import { toast } from 'sonner'
import type { McpServerDto } from '@/bindings/McpServerDto'
import { mcpServersApi } from '@/lib/api/mcp-servers'
import { McpServerFormModel } from './mcp-server-form.model'

export class McpServerSheetViewmodel {
  public form = new McpServerFormModel()
  public isOpen = false
  public isSaving = false
  private projectId: string | null = null
  private readonly onSaved?: (server: McpServerDto) => Promise<void> | void

  constructor(onSaved?: (server: McpServerDto) => Promise<void> | void) {
    this.onSaved = onSaved
    makeAutoObservable(this, {}, { autoBind: true })
  }

  public get title() {
    return this.form.isNew ? 'New MCP server' : 'Edit MCP server'
  }

  public get description() {
    return this.form.isNew
      ? 'Add a configured MCP server for this project.'
      : 'Update configuration for this MCP server.'
  }

  public get actionLabel() {
    return this.form.isNew ? 'Create server' : 'Save server'
  }

  public openForCreate(projectId: string) {
    this.projectId = projectId
    this.form.reset()
    this.isOpen = true
  }

  public openForEdit(server: McpServerDto) {
    this.projectId = server.project_id
    this.form.setFromDto(server)
    this.isOpen = true
  }

  public setOpen(isOpen: boolean) {
    if (isOpen) return
    if (this.isSaving) return
    this.isOpen = false
  }

  public async save() {
    if (!this.projectId || !this.form.isValid || this.isSaving) return

    this.isSaving = true
    try {
      const server = this.form.isNew
        ? await mcpServersApi.create(this.form.toCreatePayload(this.projectId))
        : await mcpServersApi.update(this.form.id!, this.form.toPatchPayload())

      await this.onSaved?.(server)
      this.isOpen = false
      toast.success(this.form.isNew ? 'MCP server created.' : 'MCP server updated.')
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Unable to save MCP server.')
    } finally {
      this.isSaving = false
    }
  }
}
