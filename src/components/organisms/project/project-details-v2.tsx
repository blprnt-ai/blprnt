import { Input } from '@/components/atoms/input'
import { Section } from '@/components/organisms/page/section'
import { SectionField } from '@/components/organisms/page/section-field'
import { useBlprntConfig } from '@/lib/utils/blprnt-config'
import { cn } from '@/lib/utils/cn'
import { useProjectEditorViewModel } from './project-editor.viewmodel'
import { WorkingDirectoriesV2 } from './working-directories-v2'

export const ProjectDetailsV2 = () => {
  const viewmodel = useProjectEditorViewModel()
  const config = useBlprntConfig()

  const handleNameChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    viewmodel.setName(e.target.value)
  }

  return (
    <>
      <Section>
        <SectionField title="Name">
          <Input className="w-full" data-tour="project-name" value={viewmodel.name} onChange={handleNameChange} />
          <div className={cn('text-muted-foreground font-normal', !config.seenTour ? 'hidden' : '')}>
            Give your project a name that will help you identify it in the future.
          </div>
        </SectionField>
      </Section>
      <WorkingDirectoriesV2 />
    </>
  )
}
