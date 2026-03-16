import type { LlmModel } from '@/bindings'
import { Button } from '@/components/atoms/button'
import { Input } from '@/components/atoms/input'
import { Switch } from '@/components/atoms/switch'
import { TableCell, TableRow } from '@/components/atoms/table'
import { formatContextLength } from './models-v2.utils'

interface ImportedModelRowProps {
  model: LlmModel
  onDelete: (model: LlmModel) => void
  onToggle: (model: LlmModel) => void
  onProviderSlugChange: (model: LlmModel, providerSlug: string) => void
}

export const ImportedModelRow = ({ model, onDelete, onToggle, onProviderSlugChange }: ImportedModelRowProps) => {
  return (
    <TableRow>
      <TableCell>
        <Switch checked={model.enabled} onCheckedChange={() => void onToggle(model)} />
      </TableCell>
      <TableCell className="max-w-md truncate">{model.name}</TableCell>
      <TableCell>
        <Input
          className="h-8 min-w-48"
          value={model.provider_slug ?? ''}
          onChange={(event) => void onProviderSlugChange(model, event.target.value)}
        />
      </TableCell>
      <TableCell>{formatContextLength(model.context_length)}</TableCell>
      <TableCell>
        <Button size="sm" variant="ghost" onClick={() => void onDelete(model)}>
          Delete
        </Button>
      </TableCell>
    </TableRow>
  )
}
