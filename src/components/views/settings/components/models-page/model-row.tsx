import { AnimatePresence, motion } from 'framer-motion'
import { Brain, Link, RouteIcon } from 'lucide-react'
import { useId, useState } from 'react'
import { SwitchSmall } from '@/components/atoms/switch'
import { TableCell } from '@/components/atoms/table'
import type { ModelCatalogItem } from '@/lib/models/app.model'
import { ModelUsageCell } from './model-usage-cell'
import type { SortColumn, SortDirection } from './types'
import { parseContextLength } from './utils'

interface ModelRowProps {
  model: ModelCatalogItem
  sortBy: SortColumn
  sortDirection: SortDirection
  maxOutputPrice: number
  minOutputPrice: number
  toggleSlug: (slug: string) => void
}

export const ModelRow = ({
  model,
  sortBy,
  sortDirection,
  maxOutputPrice,
  minOutputPrice,
  toggleSlug,
}: ModelRowProps) => {
  const [isToggled, setIsToggled] = useState(model.toggledOn)
  const [isAnimating, setIsAnimating] = useState(false)
  const id = useId()

  const handleToggle = (e?: React.MouseEvent<HTMLTableRowElement>) => {
    e?.preventDefault()
    e?.stopPropagation()

    setIsToggled(!isToggled)
    setTimeout(() => {
      setIsAnimating(true)
      setTimeout(() => {
        toggleSlug(model.slug)
        setIsAnimating(false)
      }, 300)
    }, 300)
  }

  const multiplier = sortDirection === 'asc' ? -1 : 1

  return (
    <AnimatePresence mode="wait">
      <motion.tr
        key={!isAnimating || sortBy !== 'enabled' ? 'none' : 'animating'}
        animate={{ opacity: 1, y: 0 }}
        className="hover:bg-muted/50 data-[state=selected]:bg-muted border-b transition-colors"
        data-slot="table-row"
        exit={{ opacity: 0, y: isToggled ? -10 * multiplier : 10 * multiplier }}
        initial={{ opacity: 0, y: isToggled ? 10 * multiplier : -10 * multiplier }}
        onClick={handleToggle}
      >
        <TableCell>
          <SwitchSmall checked={isToggled} id={id} />
        </TableCell>
        <TableCell>{model.name}</TableCell>
        <TableCell>
          <ModelUsageCell
            maxOutputPrice={maxOutputPrice}
            minOutputPrice={minOutputPrice}
            outputPrice={model.is_free ? '0' : model.output_price}
          />
        </TableCell>
        <TableCell>{parseContextLength(model.context_length)}</TableCell>
        <TableCell>{model.supports_reasoning && <Brain className="size-4" />}</TableCell>
        <TableCell>{model.auto_router && <RouteIcon className="size-4" />}</TableCell>
        <TableCell>{model.supports_oauth && <Link className="size-4" />}</TableCell>

        <TableCell></TableCell>
      </motion.tr>
    </AnimatePresence>
  )
}
