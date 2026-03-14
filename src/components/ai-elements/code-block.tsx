import { captureException } from '@sentry/react'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import type { Element } from 'hast'
import { CheckIcon, CopyIcon } from 'lucide-react'
import { type ComponentProps, createContext, type HTMLAttributes, useContext, useEffect, useRef, useState } from 'react'
import {
  type BundledLanguage,
  type BundledTheme,
  bundledLanguages,
  createHighlighter,
  createJavaScriptRegexEngine,
  type ShikiTransformer,
  type SpecialLanguage,
} from 'shiki'
import { StreamdownContext } from 'streamdown'
import { cn } from '@/lib/utils/cn'

const PRE_TAG_REGEX = /<pre(\s|>)/

type CodeBlockProps = HTMLAttributes<HTMLDivElement> & {
  code: string
  language: BundledLanguage
  preClassName?: string
}

type CodeBlockContextType = {
  code: string
}

const CodeBlockContext = createContext<CodeBlockContextType>({
  code: '',
})

class HighlighterManager {
  private highlighter: Awaited<ReturnType<typeof createHighlighter>> | null = null
  private theme: BundledTheme | null = null
  private readonly loadedLanguages: Set<BundledLanguage> = new Set()
  private initializationPromise: Promise<void> | null = null

  private isLanguageSupported(language: string): language is BundledLanguage {
    return Object.hasOwn(bundledLanguages, language)
  }

  private getFallbackLanguage(): SpecialLanguage {
    return 'plaintext'
  }

  private async ensureHighlightersInitialized(theme: BundledTheme, language: BundledLanguage): Promise<void> {
    const jsEngine = createJavaScriptRegexEngine({ forgiving: true })

    // Check if we need to recreate highlighters due to theme change
    const needsRecreation = !this.highlighter || this.theme !== theme

    if (needsRecreation) this.loadedLanguages.clear()
    const isLanguageSupported = this.isLanguageSupported(language)
    const needsLanguageLoad = !this.loadedLanguages.has(language) && isLanguageSupported

    if (needsRecreation) {
      const langsToLoad = needsLanguageLoad
        ? [...this.loadedLanguages].concat(isLanguageSupported ? [language] : [])
        : Array.from(this.loadedLanguages)

      this.highlighter = await createHighlighter({
        engine: jsEngine,
        langs: langsToLoad.length > 0 ? langsToLoad : isLanguageSupported ? [language] : [],
        themes: [theme],
      })
      this.theme = theme
    } else if (needsLanguageLoad) {
      await this.highlighter?.loadLanguage(language)
    }

    if (needsLanguageLoad) this.loadedLanguages.add(language)
  }

  lineNumberTransformer(): ShikiTransformer {
    return {
      line(node: Element, line: number) {
        node.children.unshift({
          children: [{ type: 'text', value: String(line) }],
          properties: {
            className: ['inline-block', 'min-w-10', 'mr-4', 'text-right', 'select-none', 'text-muted-foreground'],
          },
          tagName: 'span',
          type: 'element',
        })
      },
      name: 'line-numbers',
    }
  }

  addPreClass(html: string, preClassName?: string) {
    if (!preClassName) {
      return html
    }
    return html.replace(PRE_TAG_REGEX, `<pre class="${preClassName}"$1`)
  }

  async highlightCode(
    code: string,
    language: BundledLanguage,

    preClassName?: string,
  ): Promise<string> {
    if (this.initializationPromise) await this.initializationPromise

    this.initializationPromise = this.ensureHighlightersInitialized('synthwave-84', language)
    await this.initializationPromise
    this.initializationPromise = null

    const lang = this.isLanguageSupported(language) ? language : this.getFallbackLanguage()
    const transformers = lang !== this.getFallbackLanguage() ? [this.lineNumberTransformer()] : []

    const html = this.highlighter?.codeToHtml(code, { lang, theme: 'synthwave-84', transformers })
    const lines = html
      ?.split('\n')
      // idk, some weird stuff is happening with the code block, so we need to filter it out
      .filter((line) => line.trim() !== '<span class="line"><span>```__</span></span>')
      .join('\n')

    return this.addPreClass(lines!, preClassName)
  }
}

// Create a singleton instance of the highlighter manager
const highlighterManager = new HighlighterManager()

export const CodeBlock = ({ code, language, className, children, preClassName, ...props }: CodeBlockProps) => {
  const [html, setHtml] = useState<string>('')
  const mounted = useRef(false)
  // const [, theme] = useContext(ShikiThemeContext)

  useEffect(() => {
    mounted.current = true

    highlighterManager.highlightCode(code, language, preClassName).then((html) => {
      if (mounted.current) setHtml(html)
    })

    return () => {
      mounted.current = false
    }
  }, [code, language, preClassName])

  return (
    <CodeBlockContext.Provider value={{ code }}>
      <div
        className={cn(
          'group relative w-full overflow-hidden rounded-md border bg-background text-foreground',
          className,
        )}
        {...props}
      >
        <div className="relative">
          <div
            className="hidden overflow-hidden dark:block [&>pre]:m-0 [&>pre]:bg-background! [&>pre]:p-4 [&>pre]:text-foreground! [&>pre]:text-sm [&_code]:font-mono [&_code]:text-sm"
            // biome-ignore lint/security/noDangerouslySetInnerHtml: "this is needed."
            dangerouslySetInnerHTML={{ __html: html }}
          />
          {children && <div className="absolute top-2 right-2 flex items-center gap-2">{children}</div>}
        </div>
      </div>
    </CodeBlockContext.Provider>
  )
}

export type CodeBlockCopyButtonProps = ComponentProps<'button'> & {
  onCopy?: () => void
  onError?: (error: Error) => void
  timeout?: number
}

export type CodeBlockDownloadButtonProps = ComponentProps<'button'> & {
  onDownload?: () => void
  onError?: (error: Error) => void
}

export const CodeBlockCopyButton = ({
  onCopy,
  onError,
  timeout = 2000,
  children,
  className,
  code: propCode,
  ...props
}: CodeBlockCopyButtonProps & { code?: string }) => {
  const [isCopied, setIsCopied] = useState(false)
  const timeoutRef = useRef(0)
  const { code: contextCode } = useContext(CodeBlockContext)
  const { isAnimating } = useContext(StreamdownContext)
  const code = propCode ?? contextCode

  const copyToClipboard = async () => {
    if (typeof window === 'undefined' || !navigator?.clipboard?.writeText) {
      onError?.(new Error('Clipboard API not available'))
      return
    }

    try {
      if (!isCopied) {
        writeText(code)
        setIsCopied(true)
        onCopy?.()
        timeoutRef.current = window.setTimeout(() => setIsCopied(false), timeout)
      }
    } catch (error) {
      captureException(error)
      onError?.(error as Error)
    }
  }

  useEffect(() => {
    return () => {
      window.clearTimeout(timeoutRef.current)
    }
  }, [])

  const Icon = isCopied ? CheckIcon : CopyIcon

  return (
    <button
      disabled={isAnimating}
      type="button"
      className={cn(
        'cursor-pointer p-1 text-muted-foreground transition-all hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50',
        className,
      )}
      onClick={copyToClipboard}
      {...props}
    >
      {children ?? <Icon size={14} />}
    </button>
  )
}
