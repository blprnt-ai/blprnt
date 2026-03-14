import { QuestionCard } from './question-card'

export const QuestionsBucket = ({ messageKeys }: { messageKeys: string[] }) => {
  if (messageKeys.length === 0) return null
  return (
    <div className="border border-border bg-accent rounded-md p-2 px-3 text-sm border-l-4">
      <div className="flex flex-col gap-4">
        {messageKeys.map((messageKey, index) => (
          <QuestionCard key={messageKey} isLast={index === messageKeys.length - 1} messageKey={messageKey} />
        ))}
      </div>
    </div>
  )
}
