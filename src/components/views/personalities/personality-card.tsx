import { PencilLine } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { useMemo, useState } from 'react'
import Markdown from 'react-markdown'
import { AccordionContent, AccordionItem } from '@/components/atoms/accordion'
import { Button } from '@/components/atoms/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/atoms/card'
import { useAccordionPrimitive } from '@/components/atoms/primitives/use-accordian-primitive'
import type { PersonalityViewModel } from '@/components/views/personalities/personalities.viewmodel'
import { EditPersonalityDialog } from './edit-personality-dialog'

interface PersonalityCardProps {
  personality: PersonalityViewModel
}

export const PersonalityCard = observer(({ personality }: PersonalityCardProps) => {
  const [isEditPersonalityDialogOpen, setIsEditPersonalityDialogOpen] = useState(false)
  const accordion = useAccordionPrimitive()
  const value = useMemo(() => `${personality.id.toString()}-${personality.name}`, [personality.id, personality.name])

  const handleClick = () => accordion.setValue(accordion.value === value ? '' : value)

  return (
    <>
      <AccordionItem className="border-none" value={value}>
        <Card className="cursor-pointer" onClick={handleClick}>
          <CardHeader>
            <CardTitle className="flex items-center justify-between">
              <div>{personality.name}</div>
              {personality.isUserDefined && (
                <Button
                  size="sm"
                  variant="outline"
                  onClick={(e) => {
                    e.stopPropagation()
                    setIsEditPersonalityDialogOpen(true)
                  }}
                >
                  <PencilLine size={16} />
                </Button>
              )}
            </CardTitle>
            <CardDescription>{personality.description}</CardDescription>
          </CardHeader>
          <AccordionContent className="cursor-default" onClick={(e) => e.stopPropagation()}>
            <CardContent>
              <Markdown>{personality.systemPrompt}</Markdown>
            </CardContent>
          </AccordionContent>
        </Card>
      </AccordionItem>

      {isEditPersonalityDialogOpen && (
        <EditPersonalityDialog
          isOpen={isEditPersonalityDialogOpen}
          personality={personality}
          onOpenChange={setIsEditPersonalityDialogOpen}
        />
      )}
    </>
  )
})
