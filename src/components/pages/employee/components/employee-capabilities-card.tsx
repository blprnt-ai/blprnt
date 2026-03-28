import { Sparkles } from 'lucide-react'
import { LabeledTextarea } from '@/components/molecules/labeled-textarea'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { useEmployeeViewmodel } from '../employee.viewmodel'

export const EmployeeCapabilitiesCard = () => {
  const viewmodel = useEmployeeViewmodel()
  const { employee } = viewmodel

  if (!employee) return null

  return (
    <Card className="border-border/60">
      <CardHeader>
        <CardTitle>Capabilities</CardTitle>
        <CardDescription>Capture what this employee is trusted to own, decide, and execute.</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <LabeledTextarea
          hint="Separate capabilities with commas."
          label="Capability list"
          placeholder="planning, strategy, hiring"
          value={viewmodel.capabilitiesValue}
          onChange={viewmodel.setCapabilities}
        />

        <div className="rounded-2xl border border-border/60 bg-muted/20 p-4">
          <div className="mb-2 flex items-center gap-2 text-sm font-medium">
            <Sparkles className="size-4" />
            Writing guidance
          </div>
          <p className="text-sm leading-6 text-muted-foreground">
            Keep this list action-oriented. Short phrases like “roadmapping”, “budget approval”, or “performance review”
            scan better than long sentences.
          </p>
        </div>
      </CardContent>
    </Card>
  )
}
