import { createFileRoute } from '@tanstack/react-router'
import { EmployeesPage } from '@/components/pages/employees'

export const Route = createFileRoute('/employees/')({
  component: EmployeesPage,
})
