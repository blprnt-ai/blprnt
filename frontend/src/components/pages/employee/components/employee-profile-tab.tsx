import { EmployeeHierarchyCard } from './employee-hierarchy-card'
import { EmployeeIdentityCard } from './employee-identity-card'

export const EmployeeProfileTab = () => (
  <div className="grid gap-4">
    <EmployeeIdentityCard />
    <EmployeeHierarchyCard />
  </div>
)
