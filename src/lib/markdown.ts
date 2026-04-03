import { CodeBlockLowlight } from '@tiptap/extension-code-block-lowlight'
import { Details, DetailsContent, DetailsSummary } from '@tiptap/extension-details'
import { Highlight } from '@tiptap/extension-highlight'
import { TableKit } from '@tiptap/extension-table'
import { Markdown, MarkdownManager } from '@tiptap/markdown'
import { Extension } from '@tiptap/react'
import StarterKit from '@tiptap/starter-kit'

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
import { createLowlight } from 'lowlight'

const lowlight = createLowlight()
const headingInlineMarkdown = new MarkdownManager({ extensions: [] }).instance

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

type HeadingMarkdownToken = {
  depth?: number
  text?: string
  tokens?: Array<{ escaped?: boolean; raw?: string; text?: string; type: string }>
  type: 'heading'
}

const HeadingMarkdownFallback = Extension.create({
  markdownTokenName: 'heading',
  name: 'headingMarkdownFallback',
  parseMarkdown: (token, helpers) => {
    if (token.type !== 'heading') return []

    const headingToken = token as HeadingMarkdownToken
    const content =
      headingToken.tokens && headingToken.tokens.length > 0
        ? helpers.parseInline(headingToken.tokens)
        : headingToken.text
          ? helpers.parseInline(new headingInlineMarkdown.Lexer().inlineTokens(headingToken.text))
          : []

    // Tiptap's ordered-list tokenizer can empty heading inline tokens for subsequent headings.
    return helpers.createNode('heading', { level: headingToken.depth || 1 }, content)
  },
  priority: 1000,
})

export const createMarkdownExtensions = () => [
  HeadingMarkdownFallback,
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

export const parseMarkdownDocument = (value: string) =>
  new MarkdownManager({
    extensions: createMarkdownExtensions(),
  }).parse(value)
