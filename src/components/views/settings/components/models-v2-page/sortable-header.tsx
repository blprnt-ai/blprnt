import { ArrowDown, ArrowUp, ArrowUpDown } from 'lucide-react'
import type { ReactNode } from 'react'
import { TableHead } from '@/components/atoms/table'
import { cn } from '@/lib/utils/cn'
import type { SortState } from './models-v2.viewmodel'

interface SortableHeaderProps<TColumn extends string> {
  column: TColumn
  sort: SortState<TColumn>
  onSort: (column: TColumn) => void
  className?: string
  children: ReactNode
}

export const SortableHeader = <TColumn extends string>({
  column,
  sort,
  onSort,
  className,
  children,
}: SortableHeaderProps<TColumn>) => {
  const isActive = sort.column === column
  const Icon = !isActive ? ArrowUpDown : sort.direction === 'asc' ? ArrowUp : ArrowDown

  return (
    <TableHead
      className={cn('cursor-pointer select-none hover:bg-muted/50 transition-colors', className)}
      onClick={() => onSort(column)}
    >
      <div className="flex items-center gap-1.5">
        {children}
        <Icon className={cn('size-3.5', !isActive && 'text-muted-foreground/50')} />
      </div>
    </TableHead>
  )
}