import { Loader2, Plus, RefreshCw, Save, Search, Trash2 } from 'lucide-react'
import { useState } from 'react'
import type { MemorySearchResultItem } from '@/bindings'
import { Button } from '@/components/atoms/button'
import { Input } from '@/components/atoms/input'
import { ScrollArea } from '@/components/atoms/scroll-area'
import { Textarea } from '@/components/atoms/textarea'
import { basicToast } from '@/components/atoms/toaster'
import { Section } from '@/components/organisms/page/section'
import { SectionField } from '@/components/organisms/page/section-field'
// eslint-disable-next-line
import { tauriMemoryApi } from '@/lib/api/tauri/memory.api'
import { cn } from '@/lib/utils/cn'

interface ProjectMemoryFilesProps {
  projectId: string
}

export const ProjectMemoryFiles = ({ projectId }: ProjectMemoryFilesProps) => {
  const [createContent, setCreateContent] = useState('')
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<MemorySearchResultItem[]>([])
  const [selectedResultIndex, setSelectedResultIndex] = useState<number | null>(null)
  const [pathInput, setPathInput] = useState('')
  const [editorContent, setEditorContent] = useState('')
  const [isSearching, setIsSearching] = useState(false)
  const [isCreating, setIsCreating] = useState(false)
  const [isReading, setIsReading] = useState(false)
  const [isSaving, setIsSaving] = useState(false)
  const [isDeleting, setIsDeleting] = useState(false)

  const selectedResult = selectedResultIndex === null ? null : (results[selectedResultIndex] ?? null)

  const handleSearch = async () => {
    setIsSearching(true)

    try {
      const result = await tauriMemoryApi.search(projectId, query, 50)

      setResults(result.memories)
      setSelectedResultIndex(null)
    } catch (error) {
      basicToast.error({
        description: error instanceof Error ? error.message : 'Unknown error',
        title: 'Failed to search memory files',
      })
    } finally {
      setIsSearching(false)
    }
  }

  const openFile = async (path: string) => {
    if (!path.trim()) return

    setIsReading(true)

    try {
      const result = await tauriMemoryApi.read(projectId, path)

      setPathInput(result.path)
      setEditorContent(result.content)
    } catch (error) {
      basicToast.error({
        description: error instanceof Error ? error.message : 'Unknown error',
        title: 'Failed to open memory file',
      })
    } finally {
      setIsReading(false)
    }
  }

  const handleCreate = async () => {
    const content = createContent.trim()
    if (!content) return

    setIsCreating(true)

    try {
      const result = await tauriMemoryApi.create(projectId, content)

      setCreateContent('')
      setPathInput(result.path)
      setEditorContent(content)
      basicToast.success({ title: 'Memory file created' })
      await handleSearch()
    } catch (error) {
      basicToast.error({
        description: error instanceof Error ? error.message : 'Unknown error',
        title: 'Failed to create memory file',
      })
    } finally {
      setIsCreating(false)
    }
  }

  const handleSave = async () => {
    if (!pathInput.trim()) return

    setIsSaving(true)

    try {
      const result = await tauriMemoryApi.update(projectId, pathInput.trim(), editorContent)

      setPathInput(result.path)
      setEditorContent(result.content)
      basicToast.success({ title: 'Memory file saved' })
      await handleSearch()
    } catch (error) {
      basicToast.error({
        description: error instanceof Error ? error.message : 'Unknown error',
        title: 'Failed to save memory file',
      })
    } finally {
      setIsSaving(false)
    }
  }

  const handleDelete = async () => {
    const path = pathInput.trim()
    if (!path || isDeleting) return
    if (!window.confirm(`Delete memory file '${path}'?`)) return

    setIsDeleting(true)

    try {
      await tauriMemoryApi.delete(projectId, path)

      setPathInput('')
      setEditorContent('')
      basicToast.success({ title: 'Memory file deleted' })
      await handleSearch()
    } catch (error) {
      basicToast.error({
        description: error instanceof Error ? error.message : 'Unknown error',
        title: 'Failed to delete memory file',
      })
    } finally {
      setIsDeleting(false)
    }
  }

  return (
    <Section>
      <SectionField
        title={
          <div className="flex flex-col gap-2">
            <div>Memory Files</div>
            <div className="text-muted-foreground text-sm font-light">
              Search raw project memory files, open one into the editor, and write directly when needed.
            </div>
          </div>
        }
      >
        <div className="flex w-full flex-col gap-6">
          <div className="grid w-full gap-3">
            <div className="text-sm font-medium">Create memory content</div>
            <Textarea
              className="min-h-28 w-full"
              placeholder="Write raw memory content to append into today's managed file"
              value={createContent}
              onChange={(event) => setCreateContent(event.target.value)}
            />
            <div className="flex justify-end">
              <Button
                disabled={isCreating || createContent.trim().length === 0}
                variant="outline"
                onClick={handleCreate}
              >
                {isCreating ? <Loader2 className="size-4 animate-spin" /> : <Plus className="size-4" />}
                Create
              </Button>
            </div>
          </div>

          <div className="grid w-full gap-4 xl:grid-cols-[320px_minmax(0,1fr)]">
            <div className="flex min-h-128 flex-col gap-3 rounded-lg border border-border/50 bg-accent/20 p-3">
              <div className="text-muted-foreground text-xs font-normal">
                Search is discovery only. Open, save, and delete require an explicit relative path.
              </div>
              <div className="flex gap-2">
                <Input
                  placeholder="Search memory files"
                  value={query}
                  onChange={(event) => setQuery(event.target.value)}
                  onKeyDown={(event) => {
                    if (event.key === 'Enter') {
                      event.preventDefault()
                      void handleSearch()
                    }
                  }}
                />
                <Button disabled={isSearching} size="icon" variant="outline" onClick={() => void handleSearch()}>
                  {isSearching ? <Loader2 className="size-4 animate-spin" /> : <Search className="size-4" />}
                </Button>
              </div>

              <ScrollArea className="min-h-0 flex-1">
                <div className="flex flex-col gap-2 pr-3">
                  {results.length === 0 ? (
                    <div className="text-muted-foreground px-3 py-4 text-sm font-normal">No search results.</div>
                  ) : (
                    results.map((item, index) => (
                      <button
                        key={`${item.title}-${item.score}-${index}`}
                        type="button"
                        className={cn(
                          'flex w-full flex-col gap-1 rounded-md border border-transparent px-3 py-2 text-left transition-colors',
                          'hover:border-border/60 hover:bg-accent',
                          selectedResultIndex === index && 'border-primary/50 bg-accent',
                        )}
                        onClick={() => setSelectedResultIndex(index)}
                      >
                        <div className="truncate text-sm font-medium">{item.title}</div>
                        <div className="text-muted-foreground line-clamp-3 text-xs font-normal">{item.content}</div>
                      </button>
                    ))
                  )}
                </div>
              </ScrollArea>
            </div>

            <div className="flex min-h-128 flex-col gap-3 rounded-lg border border-border/50 bg-accent/20 p-3">
              <div className="flex flex-col gap-2">
                <div className="text-sm font-medium">Relative file path</div>
                <div className="flex gap-2">
                  <Input
                    placeholder="daily/2026-03-08.md"
                    value={pathInput}
                    onChange={(event) => setPathInput(event.target.value)}
                  />
                  <Button
                    disabled={!pathInput.trim() || isReading}
                    size="sm"
                    variant="outline"
                    onClick={() => void openFile(pathInput.trim())}
                  >
                    {isReading ? <Loader2 className="size-4 animate-spin" /> : <RefreshCw className="size-4" />}
                    Open
                  </Button>
                </div>
              </div>

              <div className="flex flex-wrap items-center justify-between gap-3">
                <div className="min-w-0">
                  <div className="truncate text-sm font-medium">{pathInput || 'No file selected'}</div>
                  {selectedResult && (
                    <div className="text-muted-foreground text-xs font-normal">
                      Score {selectedResult.score.toFixed(3)}
                    </div>
                  )}
                </div>

                <div className="flex gap-2">
                  <Button
                    disabled={!pathInput.trim() || isReading || isSaving}
                    size="sm"
                    variant="outline"
                    onClick={() => void openFile(pathInput.trim())}
                  >
                    {isReading ? <Loader2 className="size-4 animate-spin" /> : <RefreshCw className="size-4" />}
                    Reload
                  </Button>
                  <Button
                    disabled={!pathInput.trim() || isSaving}
                    size="sm"
                    variant="outline"
                    onClick={() => void handleSave()}
                  >
                    {isSaving ? <Loader2 className="size-4 animate-spin" /> : <Save className="size-4" />}
                    Save
                  </Button>
                  <Button
                    disabled={!pathInput.trim() || isDeleting}
                    size="sm"
                    variant="destructive"
                    onClick={() => void handleDelete()}
                  >
                    {isDeleting ? <Loader2 className="size-4 animate-spin" /> : <Trash2 className="size-4" />}
                    Delete
                  </Button>
                </div>
              </div>

              <Textarea
                className="min-h-112 flex-1 resize-none font-mono text-sm"
                placeholder="Open a file by relative path to edit raw content"
                value={editorContent}
                onChange={(event) => setEditorContent(event.target.value)}
              />

              {selectedResult && (
                <div className="flex flex-col gap-2 rounded-md border border-border/50 bg-background/40 p-3">
                  <div className="text-sm font-medium">Selected search result preview</div>
                  <div className="text-muted-foreground text-xs font-normal">{selectedResult.title}</div>
                  <div className="text-muted-foreground max-h-40 overflow-hidden whitespace-pre-wrap text-xs font-normal">
                    {selectedResult.content}
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      </SectionField>
    </Section>
  )
}
