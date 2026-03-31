import { RefreshCcwIcon } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { useEmployeesViewmodel } from '../employees.viewmodel'

export const EmployeeImportManifestFields = () => {
  const viewmodel = useEmployeesViewmodel()
  const selectedEmployee = viewmodel.selectedImportEmployee

  return (
    <div className="flex min-w-0 flex-1 flex-col gap-5">
      <div className="flex min-w-0 flex-col gap-2">
        <Label htmlFor="employee-import-base-url">Base URL</Label>
        <div className="flex flex-col gap-3 md:flex-row">
          <Input
            id="employee-import-base-url"
            value={viewmodel.importBaseUrl}
            onChange={(event) => viewmodel.setImportBaseUrl(event.target.value)}
          />
          <Button
            disabled={viewmodel.isLoadingImportManifest}
            type="button"
            variant="outline"
            onClick={() => void viewmodel.loadImportManifest()}
          >
            <RefreshCcwIcon className="size-4" />
            {viewmodel.isLoadingImportManifest ? 'Loading...' : 'Load manifest'}
          </Button>
        </div>
      </div>

      <div className="flex min-w-0 flex-col gap-2">
        <Label htmlFor="employee-import-slug">Employee</Label>
        <Select value={viewmodel.importSlug} onValueChange={(value) => viewmodel.setImportSlug(value ?? '')}>
          <SelectTrigger className="w-full" id="employee-import-slug">
            <SelectValue placeholder="Select employee">{selectedEmployee?.name ?? null}</SelectValue>
          </SelectTrigger>
          <SelectContent>
            {viewmodel.importEmployeeOptions.map((employee) => (
              <SelectItem key={employee.id} value={employee.id}>
                {employee.name}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        {selectedEmployee ? <p className="text-sm text-muted-foreground">{selectedEmployee.description}</p> : null}
        {viewmodel.importManifestError ? (
          <p className="text-sm text-destructive">{viewmodel.importManifestError}</p>
        ) : null}
      </div>
    </div>
  )
}
