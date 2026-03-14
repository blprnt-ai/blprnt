import type { PropsWithChildren } from 'react'

export const CodeInlineOrBlock = ({ children }: PropsWithChildren) => {
  const text = String(children ?? '')
  const multiline = text.includes('\n')
  if (multiline) return <CodeBlock>{text}</CodeBlock>
  return <code className="rounded bg-muted px-1 py-0.5 font-mono text-xs">{text}</code>
}

export const CodeBlock = ({ children }: PropsWithChildren) => {
  return (
    <pre className="overflow-x-auto rounded-md border bg-muted/50 p-3 text-xs leading-5">
      <code>{children}</code>
    </pre>
  )
}
