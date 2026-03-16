import { Eye, EyeOff, SaveIcon, Trash2Icon } from 'lucide-react'
import { Fragment } from 'react'
import type { Provider } from '@/bindings'
import { Button } from '@/components/atoms/button'
import { Field, FieldGroup, FieldLabel, FieldLegend, FieldSeparator, FieldSet } from '@/components/atoms/field'
import { Input } from '@/components/atoms/input'
import { InputGroup, InputGroupAddon, InputGroupInput } from '@/components/atoms/input-group'
import { Section } from '@/components/organisms/page/section'
import { SectionField } from '@/components/organisms/page/section-field'
import { newProviderId } from '@/lib/utils/default-models'
import { useProvidersPageViewmodel } from './providers-page-viewmodel'

export const CustomProviders = () => {
  const viewmodel = useProvidersPageViewmodel()
  const providers: Array<{ label: string; provider: Provider; supportsBaseUrl: boolean }> = [
    { label: 'OpenRouter', provider: 'open_router', supportsBaseUrl: false },
    { label: 'OpenAI', provider: 'openai', supportsBaseUrl: true },
    { label: 'Anthropic', provider: 'anthropic', supportsBaseUrl: true },
  ]

  return (
    <Section>
      <SectionField
        title={
          <div className="flex flex-col gap-2">
            <div>Custom Providers</div>
            <div className="text-muted-foreground text-sm font-light">
              Configure your OpenRouter, OpenAI, and Anthropic API keys.
            </div>
          </div>
        }
      >
        <FieldGroup>
          {providers.map(({ label, provider, supportsBaseUrl }, idx) => (
            <Fragment key={provider}>
              <FieldSet>
                <FieldLegend>{label}</FieldLegend>
                {provider !== 'open_router' && (
                  <div className="text-muted-foreground text-sm font-light">
                    blprnt will prefer to use this {label} api key over the{' '}
                    {provider === 'anthropic' ? 'Claude' : 'OpenAI'} subscription.
                  </div>
                )}

                <FieldGroup>
                  <Field orientation="horizontal">
                    <FieldLabel className="whitespace-nowrap min-w-20">API Key:</FieldLabel>
                    <InputGroup>
                      <InputGroupInput
                        type={viewmodel.apiKeyVisibility(provider) ? 'text' : 'password'}
                        value={viewmodel.apiKey(provider)}
                        onChange={(e) => viewmodel.setApiKey(provider, e.target.value)}
                      />
                      <InputGroupAddon align="inline-end">
                        {viewmodel.apiKeyVisibility(provider) ? (
                          <Eye className="cursor-pointer" onClick={() => viewmodel.toggleApiKeyVisibility(provider)} />
                        ) : (
                          <EyeOff
                            className="cursor-pointer"
                            onClick={() => viewmodel.toggleApiKeyVisibility(provider)}
                          />
                        )}
                      </InputGroupAddon>
                    </InputGroup>
                  </Field>
                  {supportsBaseUrl && (
                    <Field orientation="horizontal">
                      <FieldLabel className="whitespace-nowrap min-w-20">Base URL:</FieldLabel>
                      <Input
                        value={viewmodel.baseUrl(provider)}
                        onChange={(e) => viewmodel.setBaseUrl(provider, e.target.value)}
                      />
                    </Field>
                  )}
                  <Field className="justify-end" orientation="horizontal">
                    <Button size="sm" variant="outline" onClick={() => viewmodel.saveProvider(provider)}>
                      <SaveIcon className="size-4" />
                    </Button>
                    <Button
                      disabled={viewmodel.providerId(provider) === newProviderId}
                      size="sm"
                      variant="destructive"
                      onClick={() => viewmodel.deleteProvider(provider)}
                    >
                      <Trash2Icon className="size-4" />
                    </Button>
                  </Field>
                </FieldGroup>
              </FieldSet>

              {idx < providers.length - 1 && <FieldSeparator />}
            </Fragment>
          ))}
        </FieldGroup>
      </SectionField>
    </Section>
  )
}
