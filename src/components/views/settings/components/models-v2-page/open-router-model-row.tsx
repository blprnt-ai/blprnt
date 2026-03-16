import { Button } from '@/components/atoms/button'
import { TableCell, TableRow } from '@/components/atoms/table'
import { ModelUsageCell } from './model-usage-cell'
import { formatContextLength } from './models-v2.utils'
import type { OpenRouterModel } from './models-v2.viewmodel'

interface OpenRouterModelRowProps {
  model: OpenRouterModel
  imported: boolean
  minPromptPrice: number
  maxPromptPrice: number
  onImport: (model: OpenRouterModel) => void
}

export const OpenRouterModelRow = ({
  model,
  imported,
  minPromptPrice,
  maxPromptPrice,
  onImport,
}: OpenRouterModelRowProps) => {
  return (
    <TableRow>
      <TableCell className="max-w-md truncate">{model.name}</TableCell>
      <TableCell>
        <ModelUsageCell
          maxOutputPrice={maxPromptPrice}
          minOutputPrice={minPromptPrice}
          outputPrice={model.pricing.prompt}
        />
      </TableCell>
      <TableCell>{formatContextLength(model.context_length)}</TableCell>
      <TableCell>
        <Button
          disabled={imported}
          size="sm"
          variant={imported ? 'secondary' : 'outline'}
          onClick={() => void onImport(model)}
        >
          {imported ? 'Imported' : 'Import'}
        </Button>
      </TableCell>
    </TableRow>
  )
}
