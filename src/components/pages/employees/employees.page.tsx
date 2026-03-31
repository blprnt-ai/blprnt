import { DownloadIcon } from 'lucide-react'
import { Page } from '@/components/layouts/page'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { EmployeesDirectory } from './components/employees-directory'
import { useEmployeesViewmodel } from './employees.viewmodel'

export const EmployeesPage = () => {
  const viewmodel = useEmployeesViewmodel()

  return (
    <Page className="overflow-y-auto p-1 pr-2">
      <div className="flex flex-col gap-3">
        <Card>
          <CardHeader>
            <div className="flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
              <div className="space-y-1">
                <CardTitle>Employees</CardTitle>
                <CardDescription>
                  Browse the people and agents in your workspace, then open one to review or edit its configuration.
                </CardDescription>
              </div>

              <form
                className="grid w-full gap-3 rounded-lg border p-3 md:max-w-md"
                onSubmit={(event) => {
                  event.preventDefault()
                  void viewmodel.importEmployee()
                }}
              >
                <div className="grid gap-2">
                  <Label htmlFor="employee-import-slug">Import employee slug</Label>
                  <Input
                    id="employee-import-slug"
                    placeholder="company/employee-slug"
                    value={viewmodel.importSlug}
                    onChange={(event) => viewmodel.setImportSlug(event.target.value)}
                  />
                </div>

                <div className="grid gap-2 text-sm">
                  <label className="flex items-start gap-2" htmlFor="employee-import-force">
                    <input
                      checked={viewmodel.importForce}
                      className="mt-0.5 size-4"
                      id="employee-import-force"
                      type="checkbox"
                      onChange={(event) => viewmodel.setImportForce(event.target.checked)}
                    />
                    <span>Force import when the employee already exists.</span>
                  </label>

                  <label className="flex items-start gap-2" htmlFor="employee-import-skip-duplicate-skills">
                    <input
                      checked={viewmodel.importSkipDuplicateSkills}
                      className="mt-0.5 size-4"
                      id="employee-import-skip-duplicate-skills"
                      type="checkbox"
                      onChange={(event) => viewmodel.setImportSkipDuplicateSkills(event.target.checked)}
                    />
                    <span>Skip duplicate skills during import.</span>
                  </label>

                  <label className="flex items-start gap-2" htmlFor="employee-import-force-skills">
                    <input
                      checked={viewmodel.importForceSkills}
                      className="mt-0.5 size-4"
                      id="employee-import-force-skills"
                      type="checkbox"
                      onChange={(event) => viewmodel.setImportForceSkills(event.target.checked)}
                    />
                    <span>Force skill import updates.</span>
                  </label>
                </div>

                <div className="flex items-center justify-between gap-3">
                  <p className="text-xs text-muted-foreground">Imports use the same backend flow as the CLI employee import.</p>
                  <Button disabled={!viewmodel.canImport} type="submit">
                    <DownloadIcon className="size-4" />
                    {viewmodel.isImporting ? 'Importing...' : 'Import'}
                  </Button>
                </div>
              </form>
            </div>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              <p className="text-sm text-muted-foreground">
                {viewmodel.employees.length} {viewmodel.employees.length === 1 ? 'employee' : 'employees'}
              </p>

              {viewmodel.errorMessage ? <p className="text-sm text-destructive">{viewmodel.errorMessage}</p> : null}
            </div>
          </CardContent>
        </Card>

        <EmployeesDirectory />
      </div>
    </Page>
  )
}
