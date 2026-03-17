import { EmptyState } from '@/components/pages/issue/components/empty-state'
import { useEmployeesViewmodel } from '../employees.viewmodel'
import { EmployeeListItem } from './employee-list-item'

export const EmployeesDirectory = () => {
  const viewmodel = useEmployeesViewmodel()

  if (viewmodel.employees.length === 0) {
    return (
      <EmptyState
        description="Employees will appear here once they are added to your workspace."
        title="No employees yet"
      />
    )
  }

  return (
    <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
      {viewmodel.employees.map((employee) => (
        <EmployeeListItem key={employee.id} employee={employee} />
      ))}
    </div>
  )
}
