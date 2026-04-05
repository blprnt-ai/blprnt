import { createFileRoute } from '@tanstack/react-router'
import { EmployeePage } from '@/components/pages/employee'
import { AppModel } from '@/models/app.model'

export const Route = createFileRoute('/employees/$employeeId/')({
  component: EmployeePage,
  staticData: {
    breadcrumb: ({ employeeId }: Record<string, string>) =>
      AppModel.instance.resolveEmployeeName(employeeId) ?? `Employee ${employeeId.slice(0, 8)}`,
  },
})
