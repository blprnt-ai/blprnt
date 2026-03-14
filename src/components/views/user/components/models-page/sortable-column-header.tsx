import { TableHead } from '@/components/atoms/table'
import { SortIcon } from './sort-icon'
import type { SortColumn, SortState } from './types'

interface SortableColumnHeaderProps {
  column: SortColumn
  sort: SortState
  onSort: (column: SortColumn) => void
  children: React.ReactNode
}

export const SortableColumnHeader = ({ column, sort, onSort, children }: SortableColumnHeaderProps) => {
  return (
    <TableHead
      className="cursor-pointer hover:bg-muted/50 transition-colors select-none"
      onClick={() => onSort(column)}
    >
      <div className="flex items-center gap-1.5">
        {children}
        <SortIcon column={column} sort={sort} />
      </div>
    </TableHead>
  )
}
