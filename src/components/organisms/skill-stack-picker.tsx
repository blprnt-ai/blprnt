import { X } from 'lucide-react'
import type { EmployeeSkillRef } from '@/bindings/EmployeeSkillRef'
import type { Skill } from '@/bindings/Skill'
import { HintedLabel } from '@/components/molecules/hinted-label'
import { Button } from '@/components/ui/button'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'

interface SkillStackPickerProps {
  availableSkills: Skill[]
  errorMessage?: string | null
  isLoading?: boolean
  maxSkills?: number
  selectedSkills: EmployeeSkillRef[]
  onSetSkillAt: (index: number, skill: Skill | null) => void
}

export const SkillStackPicker = ({
  availableSkills,
  errorMessage,
  isLoading = false,
  maxSkills = 2,
  selectedSkills,
  onSetSkillAt,
}: SkillStackPickerProps) => {
  const visibleSlotCount = selectedSkills[0] ? maxSkills : 1
  const slots = Array.from({ length: visibleSlotCount }, (_, index) => selectedSkills[index] ?? null)

  const getOptionsForSlot = (slotIndex: number) =>
    availableSkills.filter((skill) => {
      const isSelectedInOtherSlot = selectedSkills.some(
        (selectedSkill, selectedIndex) => selectedIndex !== slotIndex && selectedSkill.name === skill.name,
      )

      return !isSelectedInOtherSlot
    })

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between gap-3">
        <HintedLabel hint={`Select up to ${maxSkills} skills to inject into this employee's runtime.`}>
          Skill stack
        </HintedLabel>
        <span className="text-xs text-muted-foreground">
          {selectedSkills.length}/{maxSkills} selected
        </span>
      </div>

      {errorMessage ? <p className="text-sm text-destructive">{errorMessage}</p> : null}

      {isLoading ? <p className="text-sm text-muted-foreground">Loading skills...</p> : null}

      {!isLoading && availableSkills.length === 0 && !errorMessage ? (
        <p className="text-sm text-muted-foreground">No skills available.</p>
      ) : null}

      {!isLoading
        ? slots.map((selectedSkill, slotIndex) => {
            const options = getOptionsForSlot(slotIndex)
            const selectedName = selectedSkill?.name ?? ''
            const selectedMetadata = options.find((skill) => skill.name === selectedName) ?? null

            return (
              <div key={`skill-slot-${slotIndex}`} className="flex flex-col gap-2">
                <div className="flex items-center gap-2">
                  <Select
                    value={selectedName}
                    onValueChange={(value) => {
                      const nextSkill = options.find((skill) => skill.name === value) ?? null
                      onSetSkillAt(slotIndex, nextSkill)
                    }}
                  >
                    <SelectTrigger className="w-full">
                      <SelectValue placeholder="No skill selected">
                        {selectedMetadata?.display_name ?? null}
                      </SelectValue>
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="">No skill selected</SelectItem>
                      {options.map((skill) => (
                        <SelectItem key={skill.path} value={skill.name}>
                          {skill.display_name}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  {selectedMetadata ? (
                    <Button size="icon-sm" type="button" variant="ghost" onClick={() => onSetSkillAt(slotIndex, null)}>
                      <X className="size-4" />
                    </Button>
                  ) : null}
                </div>
                {selectedMetadata ? (
                  <p className="text-sm text-muted-foreground">{selectedMetadata.description}</p>
                ) : null}
              </div>
            )
          })
        : null}
    </div>
  )
}
