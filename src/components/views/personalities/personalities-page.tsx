import { captureException } from '@sentry/react'
import { PencilLine } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { type KeyboardEvent, useCallback, useEffect, useMemo, useRef, useState } from 'react'
import Markdown from 'react-markdown'
import { Button } from '@/components/atoms/button'
import { Input } from '@/components/atoms/input'
import { ScrollArea } from '@/components/atoms/scroll-area'
import { Separator } from '@/components/atoms/separator'
import { Skeleton } from '@/components/atoms/skeleton'
import { deletePersonalityToast as toast } from '@/components/atoms/toaster'
import { DeleteConfirmDialog } from '@/components/dialogs/delete-confirm-dialog'
import {
  PersonalitiesViewModel,
  type PersonalityViewModel,
} from '@/components/views/personalities/personalities.viewmodel'
import { cn } from '@/lib/utils/cn'
import { EditPersonalityDialog } from './edit-personality-dialog'
import { NewPersonalityDialog } from './new-personality-dialog'

type OwnershipFilter = 'all' | 'system' | 'yours'

const resolveDefaultSelectionId = (personalities: PersonalityViewModel[]) => {
  const firstSystem = personalities.find((personality) => !personality.isUserDefined)
  return firstSystem?.id ?? personalities[0]?.id ?? null
}

const containsQuery = (value: string, query: string) => value.toLowerCase().includes(query)

export const PersonalitiesPage = observer(() => {
  const [newPersonalityDialogOpen, setNewPersonalityDialogOpen] = useState(false)
  const [editPersonalityDialogOpen, setEditPersonalityDialogOpen] = useState(false)
  const [deletePersonalityDialogOpen, setDeletePersonalityDialogOpen] = useState(false)
  const [selectedPersonalityId, setSelectedPersonalityId] = useState<string | null>(null)
  const [searchQuery, setSearchQuery] = useState('')
  const [ownershipFilter, setOwnershipFilter] = useState<OwnershipFilter>('all')
  const [operationError, setOperationError] = useState<string | null>(null)
  const listItemRefs = useRef(new Map<string, HTMLButtonElement>())

  const viewModel = useMemo(() => new PersonalitiesViewModel(), [])
  const personalities = viewModel.asArray

  const normalizedSearchQuery = searchQuery.trim().toLowerCase()

  const filteredPersonalities = useMemo(() => {
    return personalities.filter((personality) => {
      if (ownershipFilter === 'system' && personality.isUserDefined) return false
      if (ownershipFilter === 'yours' && !personality.isUserDefined) return false
      if (!normalizedSearchQuery) return true

      return (
        containsQuery(personality.name, normalizedSearchQuery) ||
        containsQuery(personality.description, normalizedSearchQuery) ||
        containsQuery(personality.systemPrompt, normalizedSearchQuery)
      )
    })
  }, [normalizedSearchQuery, ownershipFilter, personalities])

  const userPersonalities = useMemo(
    () => filteredPersonalities.filter((personality) => personality.isUserDefined),
    [filteredPersonalities],
  )
  const systemPersonalities = useMemo(
    () => filteredPersonalities.filter((personality) => !personality.isUserDefined),
    [filteredPersonalities],
  )

  const showSystemSection = ownershipFilter !== 'yours'
  const showUserSection = ownershipFilter !== 'system'

  const visiblePersonalities = useMemo(() => {
    const items: PersonalityViewModel[] = []
    if (showSystemSection) items.push(...systemPersonalities)
    if (showUserSection) items.push(...userPersonalities)
    return items
  }, [showSystemSection, showUserSection, systemPersonalities, userPersonalities])

  const hasAnyPersonalities = personalities.length > 0
  const hasAnyUserPersonalities = personalities.some((personality) => personality.isUserDefined)
  const hasActiveFilters = ownershipFilter !== 'all' || normalizedSearchQuery.length > 0

  const selectedPersonality = useMemo(
    () => visiblePersonalities.find((personality) => personality.id === selectedPersonalityId) ?? null,
    [visiblePersonalities, selectedPersonalityId],
  )

  useEffect(() => {
    if (visiblePersonalities.length === 0) {
      if (selectedPersonalityId !== null) setSelectedPersonalityId(null)
      return
    }

    const hasSelected = selectedPersonalityId
      ? visiblePersonalities.some((personality) => personality.id === selectedPersonalityId)
      : false
    if (hasSelected) return

    setSelectedPersonalityId(resolveDefaultSelectionId(visiblePersonalities))
  }, [visiblePersonalities, selectedPersonalityId])

  const handleCreateSuccess = useCallback((personality: PersonalityViewModel) => {
    setSelectedPersonalityId(personality.id)
    setOperationError(null)
  }, [])

  const handleDeleteSelectedPersonality = useCallback(async () => {
    if (!selectedPersonality || !selectedPersonality.isUserDefined) return

    toast.loading({ title: 'Deleting personality...' })
    setOperationError(null)

    try {
      await selectedPersonality.delete()
      toast.success({ title: 'Personality deleted successfully' })
      setDeletePersonalityDialogOpen(false)
    } catch (error) {
      captureException(error)
      setOperationError('Failed to delete personality. Try again.')
      toast.error({ title: `Failed to delete personality: ${error}` })
    }
  }, [selectedPersonality])

  const listItemClassName =
    'w-full rounded-lg border border-transparent bg-background/60 px-3.5 py-2.5 text-left transition-colors transition-shadow hover:border-border hover:bg-muted/50 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-amber-500 focus-visible:border-amber-500 data-[selected=true]:border-primary/60 data-[selected=true]:bg-muted'

  const handleListItemKeyDown = useCallback(
    (event: KeyboardEvent<HTMLButtonElement>, personalityId: string) => {
      if (visiblePersonalities.length === 0) return

      const index = visiblePersonalities.findIndex((personality) => personality.id === personalityId)
      if (index < 0) return

      let nextIndex = index
      if (event.key === 'ArrowDown') nextIndex = Math.min(index + 1, visiblePersonalities.length - 1)
      if (event.key === 'ArrowUp') nextIndex = Math.max(index - 1, 0)
      if (event.key === 'Home') nextIndex = 0
      if (event.key === 'End') nextIndex = visiblePersonalities.length - 1
      if (nextIndex === index) return

      const nextPersonality = visiblePersonalities[nextIndex]
      if (!nextPersonality) return

      event.preventDefault()
      setSelectedPersonalityId(nextPersonality.id)
      requestAnimationFrame(() => {
        listItemRefs.current.get(nextPersonality.id)?.focus()
      })
    },
    [visiblePersonalities],
  )

  const renderListSkeleton = () => (
    <div aria-hidden className="flex flex-col gap-5 p-4">
      <Skeleton className="h-9 w-full" />
      <Skeleton className="h-8 w-56" />
      <div className="space-y-2.5">
        <Skeleton className="h-4 w-20" />
        <Skeleton className="h-16 w-full" />
        <Skeleton className="h-16 w-full" />
        <Skeleton className="h-16 w-full" />
      </div>
    </div>
  )

  const renderDetailSkeleton = () => (
    <div aria-hidden className="flex flex-col gap-5 p-6">
      <div className="space-y-2">
        <Skeleton className="h-4 w-32" />
        <Skeleton className="h-9 w-72" />
        <Skeleton className="h-5 w-full max-w-2xl" />
      </div>
      <Separator />
      <div className="space-y-3">
        <Skeleton className="h-4 w-full max-w-3xl" />
        <Skeleton className="h-4 w-full max-w-3xl" />
        <Skeleton className="h-4 w-11/12 max-w-3xl" />
      </div>
    </div>
  )

  const loadingInitialState = viewModel.isLoading && !hasAnyPersonalities

  return (
    <>
      <div className="flex h-full min-h-0 flex-col gap-5">
        <div className="flex items-start justify-between gap-4">
          <div className="space-y-1">
            <p className="text-sm text-muted-foreground">Manage built-in and custom assistant behaviors.</p>
          </div>
          <Button
            aria-label="Create new personality"
            size="sm"
            variant="outline"
            onClick={() => setNewPersonalityDialogOpen(true)}
          >
            Create New
          </Button>
        </div>

        {operationError && (
          <div
            className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive"
            role="alert"
          >
            {operationError}
          </div>
        )}

        <div className="grid min-h-0 flex-1 grid-cols-1 rounded-xl border bg-background md:grid-cols-[300px_1px_minmax(0,1fr)]">
          <ScrollArea className="min-h-[260px] bg-muted/20 md:min-h-0">
            {loadingInitialState ? (
              renderListSkeleton()
            ) : (
              <div className="flex flex-col gap-5 p-4">
                <Input
                  aria-label="Search personalities"
                  placeholder="Search personalities..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                />

                <div
                  aria-label="Filter personalities"
                  className="inline-flex w-fit gap-1 rounded-md bg-muted p-1"
                  role="group"
                >
                  <Button
                    aria-pressed={ownershipFilter === 'all'}
                    size="sm"
                    variant={ownershipFilter === 'all' ? 'secondary' : 'ghost'}
                    onClick={() => setOwnershipFilter('all')}
                  >
                    All
                  </Button>
                  <Button
                    aria-pressed={ownershipFilter === 'system'}
                    size="sm"
                    variant={ownershipFilter === 'system' ? 'secondary' : 'ghost'}
                    onClick={() => setOwnershipFilter('system')}
                  >
                    System
                  </Button>
                  <Button
                    aria-pressed={ownershipFilter === 'yours'}
                    size="sm"
                    variant={ownershipFilter === 'yours' ? 'secondary' : 'ghost'}
                    onClick={() => setOwnershipFilter('yours')}
                  >
                    Yours
                  </Button>
                </div>

                {viewModel.loadError && (
                  <div
                    className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-xs text-destructive"
                    role="alert"
                  >
                    <div className="mb-2">Failed to load personalities.</div>
                    <Button size="sm" variant="outline" onClick={() => void viewModel.reload()}>
                      Retry
                    </Button>
                  </div>
                )}

                {showSystemSection && (
                  <section aria-label="System personalities" className="flex flex-col gap-2.5" role="listbox">
                    <div className="flex items-center justify-between gap-2">
                      <h2 className="text-[11px] font-semibold tracking-[0.12em] text-muted-foreground uppercase">
                        System
                      </h2>
                      <span className="text-[11px] text-muted-foreground">{systemPersonalities.length}</span>
                    </div>
                    {systemPersonalities.length > 0 ? (
                      systemPersonalities.map((personality) => {
                        const isSelected = selectedPersonalityId === personality.id
                        return (
                          <button
                            key={personality.id}
                            ref={(node) => {
                              if (node) listItemRefs.current.set(personality.id, node)
                              else listItemRefs.current.delete(personality.id)
                            }}
                            aria-selected={isSelected}
                            className={cn(listItemClassName, isSelected && 'shadow-sm')}
                            data-selected={isSelected}
                            role="option"
                            type="button"
                            onClick={() => setSelectedPersonalityId(personality.id)}
                            onKeyDown={(event) => handleListItemKeyDown(event, personality.id)}
                          >
                            <div className="text-sm font-medium leading-5">{personality.name}</div>
                            <div className="mt-0.5 line-clamp-2 text-xs leading-5 text-muted-foreground">
                              {personality.description}
                            </div>
                          </button>
                        )
                      })
                    ) : (
                      <div className="text-xs text-muted-foreground">No system personalities.</div>
                    )}
                  </section>
                )}

                {showUserSection && (
                  <section aria-label="Your personalities" className="flex flex-col gap-2.5" role="listbox">
                    <div className="flex items-center justify-between gap-2">
                      <h2 className="text-[11px] font-semibold tracking-[0.12em] text-muted-foreground uppercase">
                        Yours
                      </h2>
                      <span className="text-[11px] text-muted-foreground">{userPersonalities.length}</span>
                    </div>
                    {userPersonalities.length > 0 ? (
                      userPersonalities.map((personality) => {
                        const isSelected = selectedPersonalityId === personality.id
                        return (
                          <button
                            key={personality.id}
                            ref={(node) => {
                              if (node) listItemRefs.current.set(personality.id, node)
                              else listItemRefs.current.delete(personality.id)
                            }}
                            aria-selected={isSelected}
                            className={cn(listItemClassName, isSelected && 'shadow-sm')}
                            data-selected={isSelected}
                            role="option"
                            type="button"
                            onClick={() => setSelectedPersonalityId(personality.id)}
                            onKeyDown={(event) => handleListItemKeyDown(event, personality.id)}
                          >
                            <div className="text-sm font-medium leading-5">{personality.name}</div>
                            <div className="mt-0.5 line-clamp-2 text-xs leading-5 text-muted-foreground">
                              {personality.description}
                            </div>
                          </button>
                        )
                      })
                    ) : (
                      <div className="text-xs text-muted-foreground">
                        {hasAnyUserPersonalities
                          ? 'No user personalities match current filters.'
                          : 'No user personalities yet. Create one to get started.'}
                      </div>
                    )}
                  </section>
                )}

                {visiblePersonalities.length === 0 && (
                  <div className="text-xs text-muted-foreground">
                    {!hasAnyPersonalities
                      ? 'No personalities available yet.'
                      : hasActiveFilters
                        ? 'No personalities match current search or filter.'
                        : 'No personalities available.'}
                  </div>
                )}
              </div>
            )}
          </ScrollArea>

          <Separator className="md:hidden" />
          <Separator className="hidden md:block" orientation="vertical" />

          <ScrollArea className="min-h-[280px] md:min-h-0">
            {loadingInitialState ? (
              renderDetailSkeleton()
            ) : selectedPersonality ? (
              <div className="flex flex-col gap-5 p-6">
                <div className="flex items-start justify-between gap-4">
                  <div className="min-w-0 space-y-2">
                    <div className="text-[11px] font-semibold tracking-[0.12em] text-muted-foreground uppercase">
                      {selectedPersonality.isUserDefined ? 'Your Personality' : 'System Personality'}
                    </div>
                    <h2 className="text-2xl font-semibold tracking-tight">{selectedPersonality.name}</h2>
                    <p className="max-w-3xl text-sm leading-6 text-muted-foreground">
                      {selectedPersonality.description}
                    </p>
                  </div>

                  {selectedPersonality.isUserDefined && (
                    <div className="flex items-center gap-2">
                      <Button
                        aria-label={`Edit ${selectedPersonality.name}`}
                        size="sm"
                        variant="outline"
                        onClick={() => setEditPersonalityDialogOpen(true)}
                      >
                        <PencilLine size={16} />
                      </Button>
                      <Button
                        aria-label={`Delete ${selectedPersonality.name}`}
                        size="sm"
                        variant="destructive"
                        onClick={() => setDeletePersonalityDialogOpen(true)}
                      >
                        Delete
                      </Button>
                    </div>
                  )}
                </div>

                <Separator />

                <div className="max-w-3xl text-sm text-foreground/95 [&_blockquote]:my-4 [&_blockquote]:border-l-2 [&_blockquote]:border-border [&_blockquote]:pl-4 [&_code]:rounded [&_code]:bg-muted [&_code]:px-1 [&_code]:py-0.5 [&_h1]:mb-3 [&_h1]:mt-7 [&_h1]:text-xl [&_h1]:font-semibold [&_h1]:tracking-tight [&_h1:first-child]:mt-0 [&_h2]:mb-2 [&_h2]:mt-6 [&_h2]:text-lg [&_h2]:font-semibold [&_h3]:mb-2 [&_h3]:mt-5 [&_h3]:font-semibold [&_li]:my-1.5 [&_ol]:mb-4 [&_ol]:pl-6 [&_p]:mb-4 [&_p]:leading-7 [&_pre]:my-4 [&_pre]:overflow-x-auto [&_pre]:rounded-md [&_pre]:border [&_pre]:bg-muted/40 [&_pre]:p-3 [&_ul]:mb-4 [&_ul]:list-disc [&_ul]:pl-6">
                  <Markdown>{selectedPersonality.systemPrompt}</Markdown>
                </div>
              </div>
            ) : (
              <div className="p-6 text-sm text-muted-foreground">
                {!hasAnyPersonalities ? 'No personalities available yet.' : 'Select a personality to view details.'}
              </div>
            )}
          </ScrollArea>
        </div>
      </div>

      {newPersonalityDialogOpen && (
        <NewPersonalityDialog
          isOpen={newPersonalityDialogOpen}
          personalities={viewModel}
          onCreated={handleCreateSuccess}
          onOpenChange={setNewPersonalityDialogOpen}
        />
      )}

      {selectedPersonality && editPersonalityDialogOpen && (
        <EditPersonalityDialog
          isOpen={editPersonalityDialogOpen}
          personality={selectedPersonality}
          onOpenChange={setEditPersonalityDialogOpen}
        />
      )}

      {selectedPersonality?.isUserDefined && (
        <DeleteConfirmDialog
          isOpen={deletePersonalityDialogOpen}
          title="Delete Personality"
          description={
            <span className="flex flex-col gap-2">
              <span>Are you sure you want to delete this personality?</span>
              <span className="text-sm italic text-destructive/80">This action cannot be undone.</span>
            </span>
          }
          onCancel={() => setDeletePersonalityDialogOpen(false)}
          onConfirm={handleDeleteSelectedPersonality}
          onOpenChange={setDeletePersonalityDialogOpen}
        />
      )}
    </>
  )
})
