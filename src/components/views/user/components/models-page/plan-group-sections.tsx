import { useMemo } from 'react'
import { Section } from '@/components/organisms/page/section'
import { SectionField } from '@/components/organisms/page/section-field'
import type { ModelCatalogItem } from '@/lib/models/app.model'
import { upperFirst } from '@/lib/utils/string'
import { PlanGroupCard } from './plan-group-card'

interface PlanGroupSectionsProps {
  openRouterModels: Record<string, ModelCatalogItem[]>
}

const topProviders = [
  'openai',
  'anthropic',
  'x-ai',
  'deepseek',
  'qwen',
  'moonshotai',
  'meta-llama',
  'google',
  'cohere',
  'mistralai',
]

export const PlanGroupSections = ({ openRouterModels }: PlanGroupSectionsProps) => {
  const providers = useMemo(() => Object.keys(openRouterModels), [openRouterModels])
  const bottomProviders = useMemo(() => providers.filter((provider) => !topProviders.includes(provider)), [providers])

  const sortedProviders = useMemo(() => bottomProviders.sort(), [bottomProviders])

  // Split sortedProviders into an array of length 4 of arrays
  const groupedProviders = useMemo(() => {
    return sortedProviders.reduce((acc, provider, index) => {
      const groupIndex = index % 4
      acc[groupIndex] = [...(acc[groupIndex] || []), provider]
      return acc
    }, [] as string[][])
  }, [sortedProviders])

  return (
    <>
      {sortedProviders.length > 0 && (
        <Section>
          <SectionField
            title={
              <div className="flex gap-1.5">
                <div className="flex gap-1.5 font-bold">Available Models through OpenRouter</div>
              </div>
            }
          >
            <div className="max-w-5xl w-full">
              <div className="grid grid-cols-4 gap-4 mb-4">
                <div>
                  <PlanGroupCard isPremium models={openRouterModels.openai} title="ChatGPT (OpenAI)" />
                  <PlanGroupCard isPremium models={openRouterModels.deepseek} title="DeepSeek" />
                  <PlanGroupCard isPremium models={openRouterModels.cohere} title="Cohere" />
                  <PlanGroupCard isPremium models={openRouterModels.mistralai} title="Mistral AI" />

                  {groupedProviders.at(2)?.map((provider) => (
                    <PlanGroupCard key={provider} models={openRouterModels[provider]} title={uppcaseTitle(provider)} />
                  ))}
                </div>

                <div>
                  <PlanGroupCard isPremium models={openRouterModels.anthropic} title="Claude (Anthropic)" />
                  <PlanGroupCard isPremium models={openRouterModels.qwen} title="Qwen" />

                  {groupedProviders.at(0)?.map((provider) => (
                    <PlanGroupCard key={provider} models={openRouterModels[provider]} title={uppcaseTitle(provider)} />
                  ))}
                </div>

                <div>
                  <PlanGroupCard isPremium models={openRouterModels['x-ai']} title="Grok (X-AI)" />
                  <PlanGroupCard isPremium models={openRouterModels.google} title="Google" />

                  {groupedProviders.at(3)?.map((provider) => (
                    <PlanGroupCard key={provider} models={openRouterModels[provider]} title={uppcaseTitle(provider)} />
                  ))}
                </div>

                <div>
                  <PlanGroupCard isPremium models={openRouterModels['meta-llama']} title="Meta (Facebook)" />
                  <PlanGroupCard isPremium models={openRouterModels.amazon} title="Amazon" />
                  <PlanGroupCard isPremium models={openRouterModels.moonshotai} title="MoonshotAI" />

                  {groupedProviders.at(1)?.map((provider) => (
                    <PlanGroupCard key={provider} models={openRouterModels[provider]} title={uppcaseTitle(provider)} />
                  ))}
                </div>
              </div>
            </div>
          </SectionField>
        </Section>
      )}
    </>
  )
}

const uppcaseTitle = (title: string) => {
  return title
    .split('-')
    .map((word) => {
      if (word === 'ai') return 'AI'
      return upperFirst(word)
    })
    .join(' ')
}
