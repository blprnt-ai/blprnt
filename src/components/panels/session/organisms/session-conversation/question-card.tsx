import { MessageCircleQuestionMark } from 'lucide-react'
import { useState } from 'react'
import { Button } from '@/components/atoms/button'
import { Label } from '@/components/atoms/label'
import { RadioGroup, RadioGroupItem } from '@/components/atoms/radio-group'
import { Separator } from '@/components/atoms/separator'
import { Textarea } from '@/components/atoms/textarea'
import { useSessionPanelViewmodel } from '@/components/panels/session/session-panel.viewmodel'
import { QuestionAnswerMessageModel } from '@/lib/models/messages/question-answer-message.model'

export const QuestionCard = ({ messageKey, isLast }: { messageKey: string; isLast: boolean }) => {
  const viewmodel = useSessionPanelViewmodel()
  const message = viewmodel.getMessageByKey(messageKey)
  const [customAnswer, setCustomAnswer] = useState('')
  const [selectedOption, setSelectedOption] = useState('')

  if (!(message instanceof QuestionAnswerMessageModel)) return null

  const answer = message.answer ?? ''
  const hasAnswer = answer.trim().length > 0

  const handleSubmit = async () => {
    const value = selectedOption || customAnswer
    await viewmodel.submitAnswer(message.id, value)
  }

  const handleCustomAnswerChange = (event: React.ChangeEvent<HTMLTextAreaElement>) => {
    setSelectedOption('')
    setCustomAnswer(event.target.value)
  }

  return (
    <>
      <div className="flex flex-col gap-2 mb-1">
        <div className="flex items-center gap-2 mb-2 text-foreground">
          <MessageCircleQuestionMark className="size-4 text-primary/80" />
          {message.question}
        </div>

        {!hasAnswer && (
          <div className="flex flex-col items-end gap-2 ml-4">
            <div className="text-sm text-muted-foreground/80 mb-4 text-left w-full">{message.details}</div>

            {message.options.length > 0 && (
              <RadioGroup className="w-full mb-2" value={selectedOption} onValueChange={setSelectedOption}>
                {message.options.map((option) => (
                  <div key={option} className="flex items-center gap-3">
                    <RadioGroupItem id={`${message.id}-${option}`} value={option} />
                    <Label htmlFor={`${message.id}-${option}`}>{option}</Label>
                  </div>
                ))}
                <div className="flex items-start gap-3 w-full pr-4">
                  <Textarea
                    className="w-full h-8"
                    placeholder="Write your own answer"
                    value={customAnswer}
                    onChange={handleCustomAnswerChange}
                  />
                </div>
              </RadioGroup>
            )}

            {message.options.length === 0 && (
              <Textarea
                className="w-full h-8"
                placeholder="Write your own answer"
                value={customAnswer}
                onChange={handleCustomAnswerChange}
              />
            )}

            <Button
              className="w-xs"
              disabled={!selectedOption && !customAnswer.trim()}
              size="sm"
              onClick={handleSubmit}
            >
              Submit
            </Button>
          </div>
        )}

        {hasAnswer && (
          <div className="text-sm text-muted-foreground">
            Answer: <span className="text-foreground">{answer}</span>
          </div>
        )}
      </div>

      {!isLast && <Separator className="my-2" />}
    </>
  )
}
