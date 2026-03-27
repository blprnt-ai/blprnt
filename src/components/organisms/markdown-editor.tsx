import { CodeBlockLowlight } from '@tiptap/extension-code-block-lowlight'
import { Details, DetailsContent, DetailsSummary } from '@tiptap/extension-details'
import { Highlight } from '@tiptap/extension-highlight'
import { TableKit } from '@tiptap/extension-table'
import { Markdown } from '@tiptap/markdown'
import { EditorContent, useEditor } from '@tiptap/react'
import StarterKit from '@tiptap/starter-kit'
import { createLowlight } from 'lowlight'
import { Bold, Heading1, Heading2, Heading3, Italic, List, ListOrdered, Minus, Quote, SquareCode } from 'lucide-react'
import { type ReactNode, useEffect, useMemo, useRef, useState } from 'react'
import { Button } from '@/components/ui/button'
import { CopyButton } from '@/components/ui/copy-button'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { cn } from '@/lib/utils'

import 'highlight.js/styles/agate.css'
import bash from 'highlight.js/lib/languages/bash'
import c from 'highlight.js/lib/languages/c'
import cpp from 'highlight.js/lib/languages/cpp'
import css from 'highlight.js/lib/languages/css'
import elixir from 'highlight.js/lib/languages/elixir'
import erlang from 'highlight.js/lib/languages/erlang'
import go from 'highlight.js/lib/languages/go'
import haskell from 'highlight.js/lib/languages/haskell'
import java from 'highlight.js/lib/languages/java'
import js from 'highlight.js/lib/languages/javascript'
import json from 'highlight.js/lib/languages/json'
import kotlin from 'highlight.js/lib/languages/kotlin'
import md from 'highlight.js/lib/languages/markdown'
import php from 'highlight.js/lib/languages/php'
import tps from 'highlight.js/lib/languages/powershell'
import python from 'highlight.js/lib/languages/python'
import ruby from 'highlight.js/lib/languages/ruby'
import rust from 'highlight.js/lib/languages/rust'
import scala from 'highlight.js/lib/languages/scala'
import sql from 'highlight.js/lib/languages/sql'
import swift from 'highlight.js/lib/languages/swift'
import ts from 'highlight.js/lib/languages/typescript'
import xml from 'highlight.js/lib/languages/xml'
import yaml from 'highlight.js/lib/languages/yaml'
import { restoreSingleLineBreaks } from '@/lib/line-breaks'

const lowlight = createLowlight()
// register only what you need to keep bundle small
lowlight.register('bash', bash)
lowlight.register('c', c)
lowlight.register('cpp', cpp)
lowlight.register('css', css)
lowlight.register('elixir', elixir)
lowlight.register('erlang', erlang)
lowlight.register('go', go)
lowlight.register('haskell', haskell)
lowlight.register('java', java)
lowlight.register('javascript', js)
lowlight.register('js', js)
lowlight.register('json', json)
lowlight.register('kotlin', kotlin)
lowlight.register('markdown', md)
lowlight.register('md', md)
lowlight.register('php', php)
lowlight.register('powershell', tps)
lowlight.register('python', python)
lowlight.register('ruby', ruby)
lowlight.register('rust', rust)
lowlight.register('scala', scala)
lowlight.register('sh', bash)
lowlight.register('sql', sql)
lowlight.register('swift', swift)
lowlight.register('ts', ts)
lowlight.register('typescript', ts)
lowlight.register('xml', xml)
lowlight.register('yaml', yaml)

interface MarkdownEditorProps {
  value: string
  onChange: (value: string) => void
  placeholder?: string
  showPreview?: boolean
  className?: string
  dataTour?: string
}

const markdownContentClassName = cn(
  'w-full max-w-none text-sm',
  '[&_p:first-child]:mt-0 [&_p:last-child]:mb-0',
  '[&_h1]:mb-3 [&_h1]:text-2xl [&_h1]:font-bold',
  '[&_h2]:mb-3 [&_h2]:text-xl [&_h2]:font-semibold',
  '[&_h3]:mb-2 [&_h3]:text-lg [&_h3]:font-medium',
  '[&_ul]:list-disc [&_ul]:pl-6',
  '[&_ol]:list-decimal [&_ol]:pl-6',
  '[&_blockquote]:my-4 [&_blockquote]:border-l-2 [&_blockquote]:border-primary/40 [&_blockquote]:pl-4 [&_blockquote]:italic [&_blockquote]:text-muted-foreground',
  '[&_hr]:my-4 [&_hr]:border-border',
  '[&_code]:rounded-sm [&_code]:bg-background/70 [&_code]:px-1.5 [&_code]:py-0.5 [&_code]:font-mono [&_code]:text-[0.9em]',
  '[&_pre]:my-4 [&_pre]:overflow-x-auto [&_pre]:rounded-md [&_pre]:border [&_pre]:border-primary/20 [&_pre]:bg-background/80 [&_pre]:p-4',
  '[&_pre_code]:bg-transparent [&_pre_code]:p-0',
)

const createEditorExtensions = () => [
  StarterKit.configure({
    codeBlock: false,
    hardBreak: { keepMarks: false },
    link: {
      autolink: false,
      linkOnPaste: false,
      openOnClick: false,
      shouldAutoLink: () => false,
    },
  }),
  CodeBlockLowlight.configure({ lowlight }),
  Markdown,
  Details,
  DetailsSummary,
  DetailsContent,
  TableKit,
  Highlight,
]

interface MarkdownEditorPreviewProps {
  value: string
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

export const MarkdownEditorPreview = ({ value }: MarkdownEditorPreviewProps) => {
  const previewExtensions = useMemo(() => createEditorExtensions(), [])

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
        class: cn('min-h-[220px] rounded-md px-4 py-3', markdownContentClassName),
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
  dataTour,
}: MarkdownEditorProps) => {
  const [error, setError] = useState<string | null>(null)
  const [isEmpty, setIsEmpty] = useState(false)
  const lastMarkdownRef = useRef(value)
  const isApplyingExternalValueRef = useRef(false)

  const editorExtensions = useMemo(() => createEditorExtensions(), [])

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
          'min-h-[220px] h-full rounded-b-md border border-t-0 border-primary/20 bg-accent px-4 py-3 outline-none overflow-y-auto',
          markdownContentClassName,
        ),
        'data-tour': dataTour ?? '',
        role: 'textbox',
      },
    },
    extensions: editorExtensions,
    onPaste: (event) => {
      console.log('onPaste', event)
    },
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
    <div className={cn('h-full w-full', className)}>
      {error && <div className="error">{error}</div>}

      <div className="flex justify-between items-center rounded-t-md border border-primary/20 bg-accent/60 px-2.5 py-2">
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

      <div className="relative min-h-0 h-[calc(100%-3rem)]">
        {editor ? <EditorContent className="h-full" editor={editor} /> : <div>Loading editor…</div>}
        {isEmpty && (
          <div className="pointer-events-none absolute left-4 top-3 text-sm text-muted-foreground">{placeholder}</div>
        )}
      </div>

      {showPreview && <MarkdownEditorPreview value={value} />}
    </div>
  )
}
