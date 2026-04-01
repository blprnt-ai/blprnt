import { observer } from 'mobx-react-lite'
import { SkillStackPicker } from '@/components/organisms/skill-stack-picker'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { useEmployeeViewmodel } from '../employee.viewmodel'

export const EmployeeSkillStackCard = observer(() => {
  const viewmodel = useEmployeeViewmodel()
  const { employee } = viewmodel

  if (!employee) return null

  return (
    <Card className="border-border/60 z-20">
      <CardHeader>
        <CardTitle>Skill stack</CardTitle>
        <CardDescription>Select up to 2 skills to inject into this employee&apos;s runtime.</CardDescription>
      </CardHeader>
      <CardContent>
        <SkillStackPicker
          availableSkills={viewmodel.availableSkills}
          errorMessage={viewmodel.skillsErrorMessage}
          isLoading={viewmodel.isSkillsLoading}
          selectedSkills={employee.skill_stack}
          onSetSkillAt={viewmodel.setSkillAt}
        />
      </CardContent>
    </Card>
  )
})
