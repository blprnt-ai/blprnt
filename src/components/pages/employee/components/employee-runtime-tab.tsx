import { useEmployeeViewmodel } from '../employee.viewmodel'
import { EmployeeRuntimeCard } from './employee-runtime-card'
import { EmployeeRuntimeEmptyCard } from './employee-runtime-empty-card'
import { EmployeeSkillStackCard } from './employee-skill-stack-card'

export const EmployeeRuntimeTab = () => {
  const viewmodel = useEmployeeViewmodel()

  if (!viewmodel.showsAgentConfiguration) {
    return <EmployeeRuntimeEmptyCard />
  }

  return (
    <div className="grid gap-4">
      <EmployeeRuntimeCard />
      <EmployeeSkillStackCard />
    </div>
  )
}
