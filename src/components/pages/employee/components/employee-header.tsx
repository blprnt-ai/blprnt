import { Button } from '@/components/ui/button'
import { useEmployeeViewmodel } from '../employee.viewmodel'

export const EmployeeHeader = () => {
  const viewmodel = useEmployeeViewmodel()
  const { employee } = viewmodel

  if (!employee) return null

  return (
    <div className="flex flex-wrap items-center justify-between gap-3">
      <div>
        <h1 className="text-2xl font-medium">{employee.name || 'Untitled employee'}</h1>
        <p className="text-sm text-muted-foreground">
          Review identity, runtime settings, and capabilities for this employee.
        </p>
      </div>

      <div className="flex items-center gap-2">
        {viewmodel.isEditing ? (
          <>
            <Button type="button" variant="ghost" onClick={viewmodel.cancelEditing}>
              Cancel
            </Button>
            <Button disabled={!viewmodel.canSave} type="button" onClick={() => void viewmodel.save()}>
              {viewmodel.isSaving ? 'Saving...' : 'Save changes'}
            </Button>
          </>
        ) : (
          <Button type="button" onClick={viewmodel.startEditing}>
            Edit employee
          </Button>
        )}
      </div>
    </div>
  )
}
