import { ArrowLeftIcon, ArrowRightIcon, BrainIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'
import type { ReasoningEffort } from '@/bindings/ReasoningEffort'
import { LabeledInput } from '@/components/molecules/labeled-input'
import { LabeledSelect } from '@/components/molecules/labeled-select'
import { LabeledSwitch } from '@/components/molecules/labeled-switch'
import { LabeledTextarea } from '@/components/molecules/labeled-textarea'
import { SkillStackPicker } from '@/components/organisms/skill-stack-picker'
import { SlugSelect } from '@/components/organisms/slug-select'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardFooter } from '@/components/ui/card'
import { DEFAULT_REASONING_OPTION, formatDefaultReasoningLabel, reasoningEffortOptions } from '@/lib/reasoning'
import { cn } from '@/lib/utils'
import { OnboardingStep, useOnboardingViewmodel } from './onboarding.viewmodel'
import { OnboardingCardHeader } from './onboarding-card-header'

export const CreateCeo = observer(() => {
  const viewmodel = useOnboardingViewmodel()

  useEffect(() => {
    void viewmodel.ensureSkillsLoaded()
  }, [viewmodel])

  const handleNameChange = (value: string) => {
    viewmodel.ceo.name = value
  }

  const handleHeartbeatPromptChange = (value: string) => {
    viewmodel.ceo.heartbeat_prompt = value
  }

  const handleWakeOnDemandChange = (value: boolean) => {
    viewmodel.ceo.wake_on_demand = value
  }

  const handleMaxConcurrentRunsChange = (value: string) => {
    viewmodel.ceo.max_concurrent_runs = parseInt(value, 10)
  }

  const handleHeartbeatIntervalChange = (value: string) => {
    viewmodel.ceo.heartbeat_interval_sec = parseInt(value, 10)
  }

  const handleProviderConfigChange = (slug: string | null) => {
    viewmodel.ceo.slug = slug ?? ''
  }

  const handleReasoningEffortChange = (value: string | null) => {
    viewmodel.ceo.reasoning_effort = value === DEFAULT_REASONING_OPTION ? null : (value as ReasoningEffort)
  }

  const handleSave = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()

    await viewmodel.saveCeo()
  }

  const verb = !viewmodel.ceo?.id || viewmodel.ceo?.isDirty ? 'Next' : 'Save'

  return (
    <Card className="w-full overflow-hidden">
      <form onSubmit={handleSave}>
        <OnboardingCardHeader
          icon={<BrainIcon className="size-8" />}
          subtitle="Configure your CEO's settings and connect them to the provider."
          title="Hire your CEO"
        />

        <CardContent className="space-y-5 bg-linear-to-b from-background to-muted/10 py-6">
          <div className="grid gap-5 xl:grid-cols-[minmax(0,1.1fr)_minmax(20rem,0.9fr)]">
            <FormSection title="Identity">
              <div className="space-y-5">
                <LabeledInput label="Name" value={viewmodel.ceo.name} onChange={handleNameChange} />
                <SlugSelect
                  provider={viewmodel.provider.provider}
                  slug={viewmodel.ceo.slug}
                  onChange={handleProviderConfigChange}
                />
                <LabeledSelect
                  hint="Used as the default reasoning level for the CEO's runs."
                  label="Reasoning level"
                  selectedValue={formatDefaultReasoningLabel(viewmodel.ceo.reasoning_effort)}
                  value={viewmodel.ceo.reasoning_effort ?? DEFAULT_REASONING_OPTION}
                  options={[
                    { label: formatDefaultReasoningLabel(null), value: DEFAULT_REASONING_OPTION },
                    ...reasoningEffortOptions,
                  ]}
                  onChange={handleReasoningEffortChange}
                />
                <SkillStackPicker
                  availableSkills={viewmodel.availableSkills}
                  errorMessage={viewmodel.skillsErrorMessage}
                  isLoading={viewmodel.isSkillsLoading}
                  selectedSkills={viewmodel.ceo.skill_stack}
                  onSetSkillAt={viewmodel.setCeoSkillAt}
                />
              </div>
            </FormSection>

            <FormSection title="Runtime">
              <div className="space-y-4">
                <div className="rounded-xl border border-border/70 bg-background/80 p-4">
                  <LabeledSwitch
                    inline
                    hint="If disabled, the CEO only works when scheduled rather than waking for assigned tasks."
                    label="Wake on demand"
                    value={viewmodel.ceo.wake_on_demand}
                    onChange={handleWakeOnDemandChange}
                  />
                </div>

                <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-1">
                  <div className="rounded-xl border border-border/70 bg-background/80 p-4">
                    <LabeledInput
                      hint="The number of concurrent runs the CEO can have. Minimum is 1."
                      label="Max concurrent runs"
                      size="sm"
                      value={viewmodel.ceo.max_concurrent_runs.toString()}
                      onChange={handleMaxConcurrentRunsChange}
                    />
                  </div>

                  <div className="rounded-xl border border-border/70 bg-background/80 p-4">
                    <LabeledInput
                      hint="The interval in seconds for checking in on open work."
                      label="Heartbeat interval"
                      size="sm"
                      value={viewmodel.ceo.heartbeat_interval_sec.toString()}
                      onChange={handleHeartbeatIntervalChange}
                    />
                  </div>
                </div>
              </div>
            </FormSection>
          </div>

          <FormSection title="Heartbeat prompt" tone="accent">
            <LabeledTextarea
              hint="This prompt is in addition to the blprnt heartbeat and the employee HEARTBEAT.md file."
              placeholder="We're building a dirt farm."
              value={viewmodel.ceo.heartbeat_prompt}
              label={
                <>
                  Heartbeat Prompt<span className="text-xs text-muted-foreground"> (optional)</span>
                </>
              }
              onChange={handleHeartbeatPromptChange}
            />
          </FormSection>
        </CardContent>
        <CardFooter className="border-t border-border/70 bg-muted/10 py-5">
          <div className="flex w-full items-center justify-between gap-3">
            <Button type="button" variant="ghost" onClick={() => viewmodel.setStep(OnboardingStep.Project)}>
              <ArrowLeftIcon className="size-4" /> Back
            </Button>

            <Button disabled={!viewmodel.ceo.isIdentityValid} size="lg" type="submit">
              <ArrowRightIcon className="size-4" /> {verb}
            </Button>
          </div>
        </CardFooter>
      </form>
    </Card>
  )
})

const FormSection = ({
  children,
  title,
  tone = 'default',
}: {
  children: React.ReactNode
  title: string
  tone?: 'accent' | 'default'
}) => {
  return (
    <section
      className={cn(
        'rounded-2xl border p-5 shadow-xs',
        tone === 'accent'
          ? 'border-primary/20 bg-linear-to-br from-primary/8 via-background to-background'
          : 'border-border/70 bg-card/70 backdrop-blur',
      )}
    >
      <div className="mb-4">
        <h3 className="text-sm font-semibold tracking-wide text-foreground">{title}</h3>
      </div>
      {children}
    </section>
  )
}
