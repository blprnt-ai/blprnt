import { TableHead, TableHeader, TableRow } from '@/components/atoms/table'
import { TooltipMacro } from '@/components/atoms/tooltip-macro'
import { SortableColumnHeader } from './sortable-column-header'
import type { SortColumn, SortState } from './types'

interface ModelsTableHeaderProps {
  sort: SortState
  onSort: (column: SortColumn) => void
}

export const ModelsTableHeader = ({ sort, onSort }: ModelsTableHeaderProps) => {
  return (
    <TableHeader>
      <TableRow>
        <SortableColumnHeader column="enabled" sort={sort} onSort={onSort}>
          Enabled
        </SortableColumnHeader>
        <SortableColumnHeader column="name" sort={sort} onSort={onSort}>
          Model
        </SortableColumnHeader>
        <SortableColumnHeader column="usage" sort={sort} onSort={onSort}>
          <TooltipMacro
            withDelay
            tooltip="A relative (to your plan) scale of how expensive the model is to use. Higher usage will drain your allowance faster."
          >
            <span>Usage</span>
          </TooltipMacro>
        </SortableColumnHeader>
        <SortableColumnHeader column="context" sort={sort} onSort={onSort}>
          Context Length
        </SortableColumnHeader>
        <SortableColumnHeader column="reasoning" sort={sort} onSort={onSort}>
          Reasoning
        </SortableColumnHeader>
        <SortableColumnHeader column="auto-router" sort={sort} onSort={onSort}>
          Auto Router
        </SortableColumnHeader>
        <SortableColumnHeader column="oauth" sort={sort} onSort={onSort}>
          API Supported
        </SortableColumnHeader>
        <TableHead></TableHead>
      </TableRow>
    </TableHeader>
  )
}
