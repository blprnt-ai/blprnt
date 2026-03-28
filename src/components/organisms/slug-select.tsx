import { useMemo } from 'react'
import type { Provider } from '@/bindings/Provider'
import { LabeledInput } from '../molecules/labeled-input'
import { LabeledSelect } from '../molecules/labeled-select'

export const modelOptions: Record<string, { label: string; value: string }[]> = {
  anthropic: [
    { label: 'Claude Opus 4.6', value: 'claude-4-6-opus' },
    { label: 'Claude Sonnet 4.6', value: 'claude-4-6-sonnet' },
    { label: 'Claude Haiku 4.6', value: 'claude-4-6-haiku' },
    { label: 'Claude Opus 4.5', value: 'claude-4-5-opus' },
    { label: 'Claude Sonnet 4.5', value: 'claude-4-5-sonnet' },
    { label: 'Claude Haiku 4.5', value: 'claude-4-5-haiku' },
  ],
  openai: [
    { label: 'GPT-5.4', value: 'gpt-5-4' },
    { label: 'GPT-5.4 Mini', value: 'gpt-5-4-mini' },
    { label: 'GPT-5.4 Nano', value: 'gpt-5-4-nano' },
    { label: 'GPT-5.3-Codex', value: 'gpt-5-3-codex' },
    { label: 'GPT-5.2-Codex', value: 'gpt-5-2-codex' },
    { label: 'GPT-5.2', value: 'gpt-5-2' },
    { label: 'GPT-5.1-Codex', value: 'gpt-5-1-codex' },
    { label: 'GPT-5.1', value: 'gpt-5-1-2025-11-13' },
  ],
}

const getModelOptions = (provider: Provider) => {
  switch (provider) {
    case 'anthropic':
    case 'claude_code':
      return modelOptions.anthropic
    case 'openai':
    case 'codex':
      return modelOptions.openai
    default:
      return []
  }
}

interface SlugSelectProps {
  slug: string
  onChange: (slug: string | null) => void
  provider: Provider
}

export const SlugSelect = ({ slug, onChange, provider }: SlugSelectProps) => {
  const options = useMemo(() => getModelOptions(provider), [provider])

  return (
    <div className="flex flex-col gap-2">
      {provider === 'open_router' && <LabeledInput label="Model" value={slug} onChange={onChange} />}
      {provider !== 'open_router' && (
        <LabeledSelect label="Model" options={options} selectedValue={slug} value={slug} onChange={onChange} />
      )}
    </div>
  )
}
