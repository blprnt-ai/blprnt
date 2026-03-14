import type { ToolUseResponse } from '@/bindings'

const getError = (result: ToolUseResponse | undefined) => {
  if (!result) return null

  if (result.type === 'error')
    return {
      message: result.error,
    }

  return null
}

export const createDescriptionWithError = (description: React.ReactNode, result: ToolUseResponse | undefined) => {
  const error = getError(result)
  if (!error) return description

  return (
    <div className="flex flex-col gap-1">
      <div>{description}</div>
      <span className="text-destructive/60">{error.message}</span>
    </div>
  )
}
