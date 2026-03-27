import { createFileRoute } from '@tanstack/react-router'
import { EmployeePage } from '@/components/pages/employee'

export const Route = createFileRoute('/employees/$employeeId/')({
  component: EmployeePage,
})
