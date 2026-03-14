import { open } from '@tauri-apps/plugin-dialog'
import { FolderOpen, Trash2 } from 'lucide-react'
import { Button } from '@/components/atoms/button'
import { Field } from '@/components/atoms/field'
import { InputGroup, InputGroupButton, InputGroupInput } from '@/components/atoms/input-group'
import { Section } from '@/components/organisms/page/section'
import { SectionField } from '@/components/organisms/page/section-field'
import { cn } from '@/lib/utils/cn'
import { useProjectEditorViewModel } from './project-editor.viewmodel'

export const WorkingDirectoriesV2 = () => {
  const viewmodel = useProjectEditorViewModel()

  const handleDirectoryChange = (index: number, value: string) => {
    viewmodel.changeWorkingDirectory(index, value)
  }

  const handleBrowse = async (index: number) => {
    const selected = await open({
      defaultPath: viewmodel.workingDirectories[index] ?? undefined,
      directory: true,
      multiple: false,
    })

    if (selected && typeof selected === 'string') handleDirectoryChange(index, selected)
  }

  return (
    <Section>
      <SectionField title="Folders">
        <div className={cn('text-muted-foreground font-normal', viewmodel.hasWorkingDirectories && 'mb-4.5')}>
          These are the folders the agent will be limited to. This is your sandbox for safe AI-assisted development.
        </div>

        {!viewmodel.hasWorkingDirectories ? (
          <div className="w-full space-y-2">
            <div className="text-muted-foreground">No folder(s) selected yet.</div>
            <Button
              className="w-full"
              data-tour="working-directory-browse"
              size="lg"
              variant="outline"
              onClick={() => handleBrowse(0)}
            >
              Select Folder
            </Button>
          </div>
        ) : (
          <div className="w-full space-y-2">
            {viewmodel.workingDirectories.map((workingDirectory, index) => (
              <Field key={index}>
                <InputGroup
                  className={cn(
                    'bg-accent text-muted-foreground',
                    viewmodel.isValidWorkingDirectories[index] ? 'border-input' : 'border-destructive',
                  )}
                >
                  <InputGroupInput
                    value={workingDirectory}
                    onChange={(e) => handleDirectoryChange(index, e.target.value)}
                  />
                  <InputGroupButton
                    data-tour="working-directory-browse"
                    size="icon-sm"
                    onClick={() => handleBrowse(index)}
                  >
                    <FolderOpen size={18} />
                  </InputGroupButton>
                  {viewmodel.workingDirectories.length > 1 && (
                    <Button
                      className="bg-transparent"
                      size="icon-sm"
                      variant="destructive"
                      onClick={() => viewmodel.removeWorkingDirectory(index)}
                    >
                      <Trash2 size={18} />
                    </Button>
                  )}
                </InputGroup>
              </Field>
            ))}
            <Button
              className="w-full"
              data-tour="working-directory-browse"
              size="lg"
              variant="outline"
              onClick={() => {
                handleBrowse(viewmodel.workingDirectories.length)
              }}
            >
              Add Folder
            </Button>
          </div>
        )}
      </SectionField>
    </Section>
  )
}
