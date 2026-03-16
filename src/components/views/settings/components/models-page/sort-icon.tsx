import { ArrowDown, ArrowUp, ArrowUpDown } from 'lucide-react'
import type { SortColumn, SortState } from './types'

interface SortIconProps {
  column: SortColumn
  sort: SortState
}

export const SortIcon = ({ column, sort }: SortIconProps) => {
  if (sort.column !== column) {
    return <ArrowUpDown className="size-3.5 text-muted-foreground/50" />
  }
  return sort.direction === 'asc' ? <ArrowUp className="size-3.5" /> : <ArrowDown className="size-3.5" />
}
