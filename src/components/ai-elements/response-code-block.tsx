import { type DetailedHTMLProps, type HTMLAttributes, isValidElement, useContext } from 'react'
import type { ExtraProps } from 'react-markdown'
import type { BundledLanguage } from 'shiki'
import { StreamdownContext } from 'streamdown'
import { cn } from '@/lib/utils/cn'
import { CodeBlock, CodeBlockCopyButton } from './code-block'
import { Mermaid } from './mermaid'

const LANGUAGE_REGEX = /language-([^\s]+)/

export const ResponseCodeBlock = ({
  node,
  className,
  children,
  ...props
}: DetailedHTMLProps<HTMLAttributes<HTMLElement>, HTMLElement> & ExtraProps) => {
  const inline = node?.position?.start.line === node?.position?.end.line
  const { mermaid } = useContext(StreamdownContext)
  const isCompact = className?.includes('code-compact')
  const compactTextClass = isCompact ? 'text-[11px]' : undefined

  if (inline) {
    return (
      <code
        className={cn('rounded bg-muted px-1.5 py-0.5 font-mono text-sm', compactTextClass, className)}
        data-streamdown="inline-code"
        {...props}
      >
        {children}
      </code>
    )
  }

  const match = className?.match(LANGUAGE_REGEX)
  const language = (match?.at(1) ?? '') as BundledLanguage

  let code = ''
  if (
    isValidElement(children) &&
    children.props &&
    typeof children.props === 'object' &&
    'children' in children.props &&
    typeof children.props.children === 'string'
  ) {
    code = children.props.children
  } else if (typeof children === 'string') {
    code = children
  }

  if (language === 'mermaid') {
    return (
      <div
        className={cn('group relative my-4 h-auto rounded-xl border p-4', className)}
        data-streamdown="mermaid-block"
      >
        <div className="flex items-center justify-end gap-2">
          <CodeBlockCopyButton code={code} />
        </div>

        <Mermaid chart={code} config={mermaid?.config} />
      </div>
    )
  }

  if (language === 'diff') {
    const lines = code.replace(/\n$/, '').split('\n')

    return (
      <div
        className={cn('group relative my-4 h-auto rounded-xl border bg-muted/40', className)}
        data-streamdown="diff-block"
      >
        <div className="flex items-center justify-end gap-2 p-2">
          <CodeBlockCopyButton code={code} />
        </div>
        <pre
          className={cn(
            'overflow-x-auto overflow-y-hidden font-mono text-xs px-4 pb-4 whitespace-pre',
            compactTextClass,
          )}
        >
          {lines.map((line, index) => (
            <div key={`${line}-${index}`} className={getDiffLineClass(line)}>
              {line || ' '}
            </div>
          ))}
        </pre>
      </div>
    )
  }

  return (
    <CodeBlock
      className={cn('overflow-x-auto border-t', className)}
      code={code}
      data-language={language}
      data-streamdown="code-block"
      language={language}
      preClassName={cn(
        'overflow-x-auto overflow-y-hidden font-mono text-xs p-4 bg-muted/40 whitespace-break-spaces',
        compactTextClass,
      )}
    >
      <CodeBlockCopyButton />
    </CodeBlock>
  )
}

const getDiffLineClass = (line: string) => {
  return cn(
    'whitespace-pre-wrap px-2 py-0.5 text-foreground/80 w-fit',
    line.startsWith('@@') && 'text-primary bg-primary/10',
    line.startsWith('+++') || (line.startsWith('---') && 'text-muted-foreground'),
    line.startsWith('+') && 'text-success bg-success/10',
    line.startsWith('-') && 'text-destructive bg-destructive/10',
  )
}
