import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'

export const EmployeeRuntimeEmptyCard = () => (
  <Card className="border-border/60">
    <CardHeader>
      <CardTitle>Runtime</CardTitle>
    </CardHeader>
    <CardContent className="text-sm text-muted-foreground">
      Runtime configuration is only available for agent employees.
    </CardContent>
  </Card>
)
