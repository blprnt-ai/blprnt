import { Field, FieldGroup, FieldLabel } from '@/components/atoms/field'
import { Input } from '@/components/atoms/input'
import { MarkdownEditor } from '@/components/organisms/markdown-editor'
import type { PersonalityViewModel } from '@/components/views/personalities/personalities.viewmodel'

interface PersonalityFormProps {
  personality: PersonalityViewModel
}

export const PersonalityForm = ({ personality }: PersonalityFormProps) => {
  const { name, description, systemPrompt, setName, setDescription, setSystemPrompt } = personality

  return (
    <FieldGroup>
      <Field>
        <FieldLabel>Name</FieldLabel>
        <Input placeholder="Name" type="text" value={name} onChange={(e) => setName(e.target.value)} />
      </Field>

      <Field>
        <FieldLabel>Description</FieldLabel>

        <Input value={description} onChange={(e) => setDescription(e.target.value)} />
      </Field>

      <Field>
        <FieldLabel>System Prompt</FieldLabel>

        <MarkdownEditor value={systemPrompt} onChange={(value) => setSystemPrompt(value)} />
      </Field>
    </FieldGroup>
  )
}
