import { MarkdownEditor } from '@/components/organisms/markdown-editor'
import { Section } from '@/components/organisms/page/section'
import { SectionField } from '@/components/organisms/page/section-field'
import { useProjectEditorViewModel } from '@/components/organisms/project/project-editor.viewmodel'
import { TextWithDescription } from '@/components/views/shared/project-description'
import { useBlprntConfig } from '@/lib/utils/blprnt-config'

export const ProjectAgentPrimerV2 = () => {
  const viewmodel = useProjectEditorViewModel()
  const config = useBlprntConfig()

  const handleAgentPrimerChange = (nextValue: string) => {
    viewmodel.setAgentPrimer(nextValue)
  }

  return (
    <Section>
      <SectionField
        title={
          <div>
            <div>Project Agent Primer</div>
            <div className="text-muted-foreground font-normal">(optional)</div>
          </div>
        }
      >
        <div>
          <TextWithDescription
            className="bg-accent text-muted-foreground tracking-normal w-full"
            description="This is the primer for the agent. It will be used to help the agent understand the project."
            disclosure={
              <div>
                Agent Primer
                <p>
                  An optional field used to provide background context and behavioral guidance for the AI agent. This
                  text is sent as part of the system instructions before any tasks are executed.
                </p>
                <br />
                <p>
                  You can use it to describe code conventions, architectural preferences, design principles, or any
                  special considerations you want the agent to follow.
                </p>
                <br />
                <p>
                  The content helps the agent align its reasoning and output with your specific project standards.
                  Markdown formatting is supported, so you can include headers, lists, or code blocks for clarity. A
                  well-written primer can greatly improve the consistency and quality of the agent's work.
                </p>
              </div>
            }
          />
        </div>

        <div className="h-full w-full">
          <MarkdownEditor
            className={!config.seenTour ? 'min-h-[258px]' : undefined}
            dataTour="project-agent-primer-textarea"
            placeholder={viewmodel.placeHolderAgentPrimer}
            value={viewmodel.agentPrimer}
            onChange={handleAgentPrimerChange}
          />
        </div>
      </SectionField>
    </Section>
  )
}
