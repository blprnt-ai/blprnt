import { observer } from 'mobx-react-lite'
import { LabeledTextarea } from '@/components/molecules/labeled-textarea'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Switch } from '@/components/ui/switch'
import type { McpServerFormModel } from './mcp-server-form.model'

interface McpServerFieldsProps {
  form: McpServerFormModel
}

export const McpServerFields = observer(({ form }: McpServerFieldsProps) => {
  return (
    <div className="flex flex-col gap-5">
      <div className="flex flex-col gap-2">
        <Label htmlFor="mcp-display-name">Display name</Label>
        <Input
          id="mcp-display-name"
          value={form.displayName}
          onChange={(event) => (form.displayName = event.target.value)}
        />
      </div>

      <LabeledTextarea
        label="Description"
        placeholder="Short owner-facing description for this MCP server."
        value={form.description}
        onChange={(value) => (form.description = value)}
      />

      <div className="grid gap-5 md:grid-cols-2">
        <div className="flex flex-col gap-2">
          <Label>Transport</Label>
          <Select value={form.transport} onValueChange={(value) => (form.transport = value ?? 'streamable_http')}>
            <SelectTrigger className="w-full">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="streamable_http">streamable_http</SelectItem>
              <SelectItem value="sse">sse</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div className="flex items-center justify-between gap-3 rounded-sm border border-border/70 px-3 py-2.5">
          <div className="space-y-1">
            <Label>Enabled</Label>
            <p className="text-xs text-muted-foreground">Keep this server available to runs.</p>
          </div>
          <Switch checked={form.enabled} onCheckedChange={(checked) => (form.enabled = checked)} />
        </div>
      </div>

      <div className="flex flex-col gap-2">
        <Label htmlFor="mcp-endpoint-url">Endpoint URL</Label>
        <Input
          id="mcp-endpoint-url"
          value={form.endpointUrl}
          onChange={(event) => (form.endpointUrl = event.target.value)}
        />
      </div>
    </div>
  )
})
