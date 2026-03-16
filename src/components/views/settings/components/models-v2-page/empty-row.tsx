import { TableCell, TableRow } from '@/components/atoms/table'

interface EmptyRowProps {
  colSpan: number
  message: string
}

export const EmptyRow = ({ colSpan, message }: EmptyRowProps) => (
  <TableRow>
    <TableCell className="py-8 text-center text-muted-foreground" colSpan={colSpan}>
      {message}
    </TableCell>
  </TableRow>
)