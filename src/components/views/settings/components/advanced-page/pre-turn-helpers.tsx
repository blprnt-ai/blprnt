import { InfoBox } from '@/components/atoms/boxes'
import { Field, FieldGroup, FieldLabel } from '@/components/atoms/field'
import { Switch } from '@/components/atoms/switch'
import { Section } from '@/components/organisms/page/section'
import { SectionField } from '@/components/organisms/page/section-field'
import { useAdvancedPageViewModel } from './adcanced-page-viewmodel'

export const PreTurnHelpers = () => {
  const viewmodel = useAdvancedPageViewModel()

  const handleReasoningEffortClassifierToggle = (checked: boolean) => {
    void viewmodel.persistSettings({ reasoningEffortClassifierEnabled: checked })
  }

  const handleSkillMatcherToggle = (checked: boolean) => {
    void viewmodel.persistSettings({ skillMatcherEnabled: checked })
  }

  return (
    <Section>
      <SectionField
        title={
          <div className="flex flex-col gap-2">
            <div>Pre-turn helpers</div>
            <div className="text-muted-foreground text-sm font-light">
              These global controls affect extra pre-turn processing before requests are handled.
            </div>
          </div>
        }
      >
        <FieldGroup>
          <Field orientation="horizontal">
            <div className="flex-1 flex flex-col gap-1">
              <FieldLabel htmlFor="reasoning-effort-classifier">Reasoning effort classification</FieldLabel>
              <div className="text-muted-foreground text-sm font-light">
                Uses a lightweight model to classify your request into a reasoning effort.
              </div>
              <div className="text-muted-foreground text-sm font-light">
                Disabling this feature will apply a medium reasoning effort to all requests.
              </div>
            </div>
            <Switch
              checked={viewmodel.reasoningEffortClassifierEnabled}
              id="reasoning-effort-classifier"
              onCheckedChange={handleReasoningEffortClassifierToggle}
            />
          </Field>
          <Field orientation="horizontal">
            <div className="flex-1 flex flex-col gap-1">
              <FieldLabel htmlFor="skill-matcher">Skill matching</FieldLabel>
              <div className="text-muted-foreground text-sm font-light">
                Agent skills are automatically matched to your request by a lightweight model.
              </div>
              <div className="text-muted-foreground text-sm font-light">
                Agent skills give the agent superpowers by teaching the agent best practices and techniques for your
                current task.
              </div>
              <div className="text-muted-foreground text-sm font-light">
                Disabling this feature removes the agent skill selection step and no skills will be applied to the
                request.
              </div>
            </div>
            <Switch
              checked={viewmodel.skillMatcherEnabled}
              id="skill-matcher"
              onCheckedChange={handleSkillMatcherToggle}
            />
          </Field>
        </FieldGroup>
        <InfoBox className="w-full">
          <div className="flex flex-col gap-1 mt-2">
            <div>
              Pre-turn helpers use slightly more tokens upfront but can dramatically improve the quality of the
              responses.
            </div>
          </div>
        </InfoBox>
      </SectionField>
    </Section>
  )
}
