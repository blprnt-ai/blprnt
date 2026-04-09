import { DownloadIcon, SparklesIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { useId } from 'react'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { cn } from '@/lib/utils'
import { useEmployeesViewmodel } from '../employees.viewmodel'
import { EmployeeImportManifestFields } from './employee-import-manifest-fields'

export const EmployeeImportCard = observer(() => {
  const viewmodel = useEmployeesViewmodel()
  const formId = useId()

  return (
    <Card className="border-border/60 py-0">
      <CardContent className="flex flex-col gap-5 px-5 py-5 md:px-6 xl:flex-row xl:items-end xl:gap-8">
        <form
          id={formId}
          onSubmit={(event) => {
            event.preventDefault()
            void viewmodel.importEmployee()
          }}
        >
          <div className="flex min-w-0 flex-1 flex-col gap-5">
            <div className="min-w-0 flex-1 space-y-3">
              <div className="flex items-center gap-2 text-xs uppercase tracking-[0.18em] text-muted-foreground">
                <SparklesIcon className="size-4" />
                <span>Import employee</span>
              </div>
              <EmployeeImportManifestFields />
            </div>

            <div className="flex flex-1 flex-col gap-3 md:flex-row md:flex-wrap md:items-start md:gap-x-6 md:gap-y-3 xl:pt-1">
              <ImportToggle
                checked={viewmodel.importForce}
                label="Force employee import"
                onChange={viewmodel.setImportForce}
              />
              <ImportToggle
                checked={viewmodel.importSkipDuplicateSkills}
                label="Skip duplicate skills"
                onChange={viewmodel.setImportSkipDuplicateSkills}
              />
              <ImportToggle
                checked={viewmodel.importForceSkills}
                label="Force skill updates"
                onChange={viewmodel.setImportForceSkills}
              />
            </div>
          </div>
        </form>

        <div className="flex items-end">
          <Button disabled={!viewmodel.canImport} form={formId} type="submit">
            <DownloadIcon className="size-4" />
            {viewmodel.isImporting ? 'Importing...' : 'Import'}
          </Button>
        </div>
      </CardContent>
    </Card>
  )
})

const ImportToggle = ({
  checked,

  label,
  onChange,
}: {
  checked: boolean

  label: string
  onChange: (checked: boolean) => void
}) => {
  const id = useId()

  return (
    <label
      htmlFor={id}
      className={cn(
        'flex min-w-0 flex-1 items-center gap-3 rounded-sm py-1 transition-colors md:max-w-56',
        checked && 'text-foreground',
      )}
    >
      <input
        checked={checked}
        className="size-4"
        id={id}
        type="checkbox"
        onChange={(event) => onChange(event.target.checked)}
      />
      <span className="min-w-0">
        <span className="block text-sm font-medium">{label}</span>
      </span>
    </label>
  )
}
