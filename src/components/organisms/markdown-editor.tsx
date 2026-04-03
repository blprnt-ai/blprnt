import { EditorContent, useEditor } from '@tiptap/react'
import { Bold, Heading1, Heading2, Heading3, Italic, List, ListOrdered, Minus, Quote, SquareCode } from 'lucide-react'
import { type ReactNode, useEffect, useMemo, useRef, useState } from 'react'
import { Button } from '@/components/ui/button'
import { CopyButton } from '@/components/ui/copy-button'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { createMarkdownExtensions } from '@/lib/markdown'
import { cn } from '@/lib/utils'

import 'highlight.js/styles/agate.css'
import { restoreSingleLineBreaks } from '@/lib/line-breaks'

interface MarkdownEditorProps {
  value: string
  onChange: (value: string) => void
  placeholder?: string
  showPreview?: boolean
  className?: string
  editorClassName?: string
  dataTour?: string
}

const markdownContentClassName = cn(
  'w-full max-w-none text-sm',
  '[&_p:first-child]:mt-0 [&_p:last-child]:mb-0',
  '[&_h1]:my-3 [&_h1]:text-2xl [&_h1]:font-bold [&_h1]:border-b [&_h1]:border-border/80',
  '[&_h2]:my-3 [&_h2]:text-xl [&_h2]:font-semibold',
  '[&_h3]:my-2 [&_h3]:text-lg [&_h3]:font-medium',
  '[&_ul]:list-disc [&_ul]:pl-6',
  '[&_ol]:list-decimal [&_ol]:pl-6',
  '[&_blockquote]:my-4 [&_blockquote]:border-l-2 [&_blockquote]:border-primary/40 [&_blockquote]:pl-4 [&_blockquote]:italic [&_blockquote]:text-muted-foreground',
  '[&_hr]:my-4 [&_hr]:border-border',
  '[&_code]:rounded-sm [&_code]:bg-background/70 [&_code]:px-1.5 [&_code]:py-0.5 [&_code]:font-mono [&_code]:text-[0.9em]',
  '[&_pre]:my-4 [&_pre]:overflow-x-auto [&_pre]:rounded-md [&_pre]:border [&_pre]:border-primary/20 [&_pre]:bg-background/80 [&_pre]:p-4',
  '[&_pre_code]:bg-transparent [&_pre_code]:p-0',
)

interface MarkdownEditorPreviewProps {
  value: string
  className?: string
}

interface ToolbarButtonProps {
  icon: ReactNode
  isActive?: boolean
  label: string
  onClick: () => void
}

const ToolbarButton = ({ icon, isActive = false, label, onClick }: ToolbarButtonProps) => (
  <Tooltip>
    <TooltipTrigger
      render={
        <Button
          aria-label={label}
          aria-pressed={isActive}
          className="size-7 shrink-0 px-0"
          size="xs"
          type="button"
          variant="ghost"
          onClick={onClick}
          onMouseDown={(event) => {
            event.preventDefault()
          }}
        >
          <span aria-hidden="true" className="flex items-center justify-center">
            {icon}
          </span>
        </Button>
      }
    />
    <TooltipContent>{label}</TooltipContent>
  </Tooltip>
)

export const MarkdownEditorPreview = ({ value, className }: MarkdownEditorPreviewProps) => {
  const previewExtensions = useMemo(() => createMarkdownExtensions(), [])

  const previewEditor = useEditor({
    content: value,
    contentType: 'markdown',
    coreExtensionOptions: {
      clipboardTextSerializer: {
        blockSeparator: '\n',
      },
    },
    editable: false,
    editorProps: {
      attributes: {
        class: cn('rounded-md', markdownContentClassName, className),
      },
    },
    extensions: previewExtensions,
  })

  useEffect(() => {
    if (!previewEditor) return

    try {
      previewEditor.commands.setContent(value, { contentType: 'markdown' })
    } catch {}
  }, [previewEditor, value])

  return <div>{previewEditor ? <EditorContent editor={previewEditor} /> : null}</div>
}

export const MarkdownEditor = ({
  value,
  onChange,
  placeholder = '',
  showPreview = false,
  className,
  editorClassName,
  dataTour,
}: MarkdownEditorProps) => {
  const [error, setError] = useState<string | null>(null)
  const [isEmpty, setIsEmpty] = useState(false)
  const lastMarkdownRef = useRef(value)
  const isApplyingExternalValueRef = useRef(false)

  const editorExtensions = useMemo(() => createMarkdownExtensions(), [])

  const editor = useEditor({
    content: value,
    contentType: 'markdown',
    coreExtensionOptions: {
      clipboardTextSerializer: {
        blockSeparator: '\n',
      },
    },
    editable: true,
    editorProps: {
      attributes: {
        'aria-label': 'Markdown editor',
        'aria-multiline': 'true',
        class: cn(
          'min-h-[220px] rounded-b-md border border-t-0 border-border/80 bg-background/88 px-4 py-3 outline-none overflow-y-auto',
          markdownContentClassName,
          editorClassName,
        ),
        'data-tour': dataTour ?? '',
        role: 'textbox',
      },
    },
    extensions: editorExtensions,
    onUpdate: ({ editor: currentEditor }) => {
      if (isApplyingExternalValueRef.current) return
      const markdown = currentEditor.getMarkdown()

      lastMarkdownRef.current = markdown
      onChange(markdown)
    },
  })

  useEffect(() => {
    setIsEmpty(value.trim() === '')
  }, [value])

  useEffect(() => {
    if (!editor) return
    if (value === lastMarkdownRef.current) return

    try {
      setError(null)
      isApplyingExternalValueRef.current = true
      editor.commands.setContent(value, { contentType: 'markdown' })
      lastMarkdownRef.current = value
    } catch (err) {
      setError(`Error parsing markdown: ${err instanceof Error ? err.message : String(err)}`)
    } finally {
      isApplyingExternalValueRef.current = false
    }
  }, [editor, value])

  return (
    <div className={cn('w-full', className)}>
      {error && <div className="error">{error}</div>}

      <div className="flex items-center justify-between rounded-t-md border border-border/80 bg-muted/35 px-2.5 py-2">
        <div aria-label="Markdown formatting toolbar" className="flex flex-wrap items-center gap-1.5" role="toolbar">
          <div className="flex flex-wrap items-center gap-1.5">
            <ToolbarButton
              icon={<Heading1 className="size-3.5" />}
              isActive={editor?.isActive('heading', { level: 1 })}
              label="Heading 1"
              onClick={() => editor?.chain().focus().toggleHeading({ level: 1 }).run()}
            />
            <ToolbarButton
              icon={<Heading2 className="size-3.5" />}
              isActive={editor?.isActive('heading', { level: 2 })}
              label="Heading 2"
              onClick={() => editor?.chain().focus().toggleHeading({ level: 2 }).run()}
            />
            <ToolbarButton
              icon={<Heading3 className="size-3.5" />}
              isActive={editor?.isActive('heading', { level: 3 })}
              label="Heading 3"
              onClick={() => editor?.chain().focus().toggleHeading({ level: 3 }).run()}
            />
          </div>

          <div className="h-5 w-px shrink-0 bg-primary/20" />

          <div className="flex flex-wrap items-center gap-1.5">
            <ToolbarButton
              icon={<Bold className="size-3.5" />}
              isActive={editor?.isActive('bold')}
              label="Bold"
              onClick={() => editor?.chain().focus().toggleBold().run()}
            />
            <ToolbarButton
              icon={<Italic className="size-3.5" />}
              isActive={editor?.isActive('italic')}
              label="Italic"
              onClick={() => editor?.chain().focus().toggleItalic().run()}
            />
          </div>

          <div className="h-5 w-px shrink-0 bg-primary/20" />

          <div className="flex flex-wrap items-center gap-1.5">
            <ToolbarButton
              icon={<List className="size-3.5" />}
              isActive={editor?.isActive('bulletList')}
              label="Bullet list"
              onClick={() => editor?.chain().focus().toggleBulletList().run()}
            />
            <ToolbarButton
              icon={<ListOrdered className="size-3.5" />}
              isActive={editor?.isActive('orderedList')}
              label="Ordered list"
              onClick={() => editor?.chain().focus().toggleOrderedList().run()}
            />
            <ToolbarButton
              icon={<SquareCode className="size-3.5" />}
              isActive={editor?.isActive('codeBlock')}
              label="Code block"
              onClick={() => editor?.chain().focus().toggleCodeBlock().run()}
            />
            <ToolbarButton
              icon={<Quote className="size-3.5" />}
              isActive={editor?.isActive('blockquote')}
              label="Blockquote"
              onClick={() => editor?.chain().focus().toggleBlockquote().run()}
            />
            <ToolbarButton
              icon={<Minus className="size-3.5" />}
              label="Horizontal rule"
              onClick={() => editor?.chain().focus().setHorizontalRule().run()}
            />
          </div>
        </div>
        <div className="flex flex-wrap items-center gap-1.5">
          <CopyButton content={restoreSingleLineBreaks(value)} size="xs" variant="link" />
        </div>
      </div>

      <div className="relative min-h-0">
        {editor ? <EditorContent editor={editor} /> : <div>Loading editor…</div>}
        {isEmpty && (
          <div className="pointer-events-none absolute left-4 top-3 text-sm text-muted-foreground">{placeholder}</div>
        )}
      </div>

      {showPreview && <MarkdownEditorPreview value={value} />}
    </div>
  )
}
