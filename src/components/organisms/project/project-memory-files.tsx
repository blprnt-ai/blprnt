import { invoke } from '@tauri-apps/api/core'
import { AlertCircle, CalendarDays, Clock3, FileText, FolderOpen, LoaderCircle, Save, Trash2 } from 'lucide-react'
import type { ReactNode } from 'react'
import { useCallback, useEffect, useMemo, useState } from 'react'
import { Button } from '@/components/atoms/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/atoms/dialog'
import { ScrollArea } from '@/components/atoms/scroll-area'
import {
  TreeExpander,
  TreeIcon,
  TreeLabel,
  TreeNode,
  TreeNodeContent,
  TreeNodeTrigger,
  TreeProvider,
  TreeView,
} from '@/components/atoms/tree'
import { DeleteConfirmDialog } from '@/components/dialogs/delete-confirm-dialog'
import { MarkdownEditor } from '@/components/organisms/markdown-editor'
import { cn } from '@/lib/utils/cn'

interface ProjectMemoryFilesProps {
  projectId: string
}

type MemoryTreeNode =
  | {
      type: 'directory'
      name: string
      path: string
      children: MemoryTreeNode[]
    }
  | {
      type: 'file'
      name: string
      path: string
    }

type MemoryListResult = {
  root_path: string
  nodes: MemoryTreeNode[]
}

type MemoryReadResult = {
  path: string
  content: string
}

const ROOT_NODE_ID = 'memory-root'

const StatePanel = ({
  icon,
  title,
  description,
  tone = 'default',
}: {
  icon: ReactNode
  title: string
  description: string
  tone?: 'default' | 'error'
}) => (
  <div
    className={cn(
      'flex h-full min-h-[260px] flex-col items-center justify-center rounded-xl border border-dashed px-6 py-8 text-center',
      tone === 'error'
        ? 'border-destructive/30 bg-destructive/5 text-destructive'
        : 'border-border/70 bg-background/70 text-muted-foreground',
    )}
  >
    <div
      className={cn(
        'mb-4 flex size-10 items-center justify-center rounded-full border',
        tone === 'error' ? 'border-destructive/30 bg-destructive/10' : 'border-border/70 bg-accent/40',
      )}
    >
      {icon}
    </div>
    <div className={cn('text-sm font-medium', tone === 'error' ? 'text-destructive' : 'text-foreground/90')}>
      {title}
    </div>
    <div className={cn('mt-1 max-w-md text-sm', tone === 'error' ? 'text-destructive/80' : 'text-muted-foreground')}>
      {description}
    </div>
  </div>
)

const collectExpandedDirectoryIds = (nodes: MemoryTreeNode[]): string[] => {
  const expandedIds: string[] = [ROOT_NODE_ID]

  const visit = (node: MemoryTreeNode) => {
    if (node.type !== 'directory') {
      return
    }

    expandedIds.push(`memory-directory-${node.path}`)
    node.children.forEach(visit)
  }

  nodes.forEach(visit)

  return expandedIds
}

type MemoryTreeBranchProps = {
  nodes: MemoryTreeNode[]
  selectedPath: string | null
  onSelectPath: (path: string) => void
  level: number
}

const MemoryTreeBranch = ({ nodes, selectedPath, onSelectPath, level }: MemoryTreeBranchProps) => (
  <>
    {nodes.map((node, index) => {
      const isLast = index === nodes.length - 1

      if (node.type === 'directory') {
        return (
          <TreeNode key={node.path} isLast={isLast} level={level} nodeId={`memory-directory-${node.path}`}>
            <TreeNodeTrigger className="h-8 rounded-md px-2 hover:bg-accent/70">
              <TreeExpander hasChildren />
              <TreeIcon hasChildren />
              <TreeLabel className="text-foreground/80">{node.name}</TreeLabel>
            </TreeNodeTrigger>
            <TreeNodeContent hasChildren>
              <MemoryTreeBranch
                level={level + 1}
                nodes={node.children}
                selectedPath={selectedPath}
                onSelectPath={onSelectPath}
              />
            </TreeNodeContent>
          </TreeNode>
        )
      }

      const isSelected = node.path === selectedPath

      return (
        <TreeNode key={node.path} isLast={isLast} level={level} nodeId={`memory-file-${node.path}`}>
          <TreeNodeTrigger
            className={cn(
              'h-8 rounded-md px-2 hover:bg-accent/70',
              isSelected && 'bg-primary/8 text-primary hover:bg-primary/10',
            )}
            onClick={(event) => {
              event.stopPropagation()
              onSelectPath(node.path)
            }}
          >
            <TreeExpander hasChildren={false} />
            <TreeIcon icon={<FileText className="size-4" />} />
            <TreeLabel className="text-[13px] text-foreground/75">{node.name}</TreeLabel>
          </TreeNodeTrigger>
        </TreeNode>
      )
    })}
  </>
)

export const ProjectMemoryFiles = ({ projectId }: ProjectMemoryFilesProps) => {
  const [draftContent, setDraftContent] = useState('')
  const [loadedContent, setLoadedContent] = useState('')
  const [treeResult, setTreeResult] = useState<MemoryListResult | null>(null)
  const [selectedPath, setSelectedPath] = useState<string | null>(null)
  const [loadedPath, setLoadedPath] = useState<string | null>(null)
  const [isLoadingTree, setIsLoadingTree] = useState(true)
  const [isLoadingFile, setIsLoadingFile] = useState(false)
  const [isSaving, setIsSaving] = useState(false)
  const [isDeleting, setIsDeleting] = useState(false)
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false)
  const [isSwitchDialogOpen, setIsSwitchDialogOpen] = useState(false)
  const [pendingSelectionPath, setPendingSelectionPath] = useState<string | null>(null)
  const [fileError, setFileError] = useState<string | null>(null)

  const refreshTree = useCallback(async () => {
    const result = await invoke<MemoryListResult>('memory_list', { request: { project_id: projectId } })

    setTreeResult(result)
    setSelectedPath((currentPath) => {
      if (!currentPath) {
        return currentPath
      }

      const hasSelectedPath = (nodes: MemoryTreeNode[]): boolean =>
        nodes.some((node) =>
          node.type === 'file'
            ? node.path === currentPath
            : node.path === currentPath || hasSelectedPath(node.children),
        )

      return hasSelectedPath(result.nodes) ? currentPath : null
    })
  }, [projectId])

  useEffect(() => {
    let isCancelled = false

    const loadTree = async () => {
      setIsLoadingTree(true)

      try {
        const result = await invoke<MemoryListResult>('memory_list', { request: { project_id: projectId } })

        if (isCancelled) {
          return
        }

        setTreeResult(result)
        setSelectedPath((currentPath) => {
          if (!currentPath) {
            return currentPath
          }

          const hasSelectedPath = (nodes: MemoryTreeNode[]): boolean =>
            nodes.some((node) =>
              node.type === 'file'
                ? node.path === currentPath
                : node.path === currentPath || hasSelectedPath(node.children),
            )

          return hasSelectedPath(result.nodes) ? currentPath : null
        })
      } catch {
        if (isCancelled) {
          return
        }

        setTreeResult(null)
        setSelectedPath(null)
        setLoadedPath(null)
        setLoadedContent('')
        setDraftContent('')
        setFileError(null)
      } finally {
        if (!isCancelled) {
          setIsLoadingTree(false)
        }
      }
    }

    void loadTree()

    return () => {
      isCancelled = true
    }
  }, [projectId])

  useEffect(() => {
    let isCancelled = false

    if (!selectedPath) {
      setLoadedPath(null)
      setLoadedContent('')
      setDraftContent('')
      setFileError(null)
      setIsLoadingFile(false)
      setIsSaving(false)
      return
    }

    const loadFile = async () => {
      setIsLoadingFile(true)
      setFileError(null)
      setLoadedPath(null)
      setLoadedContent('')
      setDraftContent('')

      try {
        const result = await invoke<MemoryReadResult>('memory_read', {
          request: {
            path: selectedPath,
            project_id: projectId,
          },
        })

        if (isCancelled) {
          return
        }

        setLoadedPath(result.path)
        setLoadedContent(result.content)
        setDraftContent(result.content)
      } catch (error) {
        if (isCancelled) {
          return
        }

        setLoadedPath(null)
        setLoadedContent('')
        setDraftContent('')
        setFileError(error instanceof Error ? error.message : 'Failed to load memory file.')
      } finally {
        if (!isCancelled) {
          setIsLoadingFile(false)
        }
      }
    }

    void loadFile()

    return () => {
      isCancelled = true
    }
  }, [projectId, selectedPath])

  const defaultExpandedIds = useMemo(() => collectExpandedDirectoryIds(treeResult?.nodes ?? []), [treeResult])
  const isDirty = loadedPath !== null && draftContent !== loadedContent
  const canSave = Boolean(loadedPath) && !isLoadingFile && !isSaving && isDirty
  const canDelete = Boolean(loadedPath) && !isLoadingFile && !isSaving && !isDeleting

  const loadedFileName = loadedPath?.split('/').at(-1) ?? null
  const pendingFileName = selectedPath?.split('/').at(-1) ?? null
  const pendingSwitchFileName = pendingSelectionPath?.split('/').at(-1) ?? null
  const headerFileName =
    isLoadingFile && pendingFileName
      ? `Loading ${pendingFileName}...`
      : fileError && pendingFileName
        ? pendingFileName
        : (loadedFileName ?? 'No file selected')
  const headerPathLabel =
    isLoadingFile && selectedPath
      ? `Pending ${selectedPath}`
      : fileError && selectedPath
        ? `Failed ${selectedPath}`
        : (loadedPath ?? 'No file loaded')
  const deleteDescription = (
    <span className="flex flex-col gap-2">
      <span>Delete the selected memory file?</span>
      <span className="text-sm italic text-destructive/80">This action cannot be undone.</span>
    </span>
  )
  const headerStatusLabel = isLoadingFile
    ? 'Loading file content'
    : isSaving
      ? 'Saving changes'
      : isDeleting
        ? 'Deleting file'
        : fileError
          ? 'Load failed'
          : isDirty
            ? 'Unsaved changes'
            : loadedPath
              ? 'Saved'
              : 'Awaiting file selection'
  const treeHasNodes = (treeResult?.nodes.length ?? 0) > 0
  const showEditorOverlay = Boolean(fileError) || !selectedPath || isLoadingFile || isDeleting

  const proceedWithPendingSelection = useCallback(() => {
    if (!pendingSelectionPath) {
      return
    }

    setSelectedPath(pendingSelectionPath)
    setPendingSelectionPath(null)
    setIsSwitchDialogOpen(false)
  }, [pendingSelectionPath])

  const handleSave = useCallback(async (): Promise<boolean> => {
    if (!loadedPath || !canSave) {
      return false
    }

    setIsSaving(true)
    setFileError(null)

    try {
      const result = await invoke<MemoryReadResult>('memory_update', {
        request: {
          content: draftContent,
          path: loadedPath,
          project_id: projectId,
        },
      })

      setLoadedPath(result.path)
      setLoadedContent(result.content)
      setDraftContent(result.content)
      return true
    } catch (error) {
      setFileError(error instanceof Error ? error.message : 'Failed to save memory file.')
      return false
    } finally {
      setIsSaving(false)
    }
  }, [canSave, draftContent, loadedPath, projectId])

  const handleSelectPath = useCallback(
    (path: string) => {
      if (path === selectedPath) {
        return
      }

      if (isDirty) {
        setPendingSelectionPath(path)
        setIsSwitchDialogOpen(true)
        return
      }

      setSelectedPath(path)
    },
    [isDirty, selectedPath],
  )

  const handleSwitchDialogSave = async () => {
    const didSave = await handleSave()

    if (didSave) {
      proceedWithPendingSelection()
    }
  }

  const handleSwitchDialogDiscard = () => {
    if (!pendingSelectionPath) {
      return
    }

    setDraftContent(loadedContent)
    proceedWithPendingSelection()
  }

  const handleSwitchDialogCancel = () => {
    if (isSaving) {
      return
    }

    setPendingSelectionPath(null)
    setIsSwitchDialogOpen(false)
  }

  const handleSwitchDialogOpenChange = (isOpen: boolean) => {
    if (isSaving) {
      return
    }

    setIsSwitchDialogOpen(isOpen)

    if (!isOpen) {
      setPendingSelectionPath(null)
    }
  }

  const handleDelete = async () => {
    if (!loadedPath || !canDelete) {
      return
    }

    setIsDeleting(true)
    setFileError(null)

    try {
      await invoke('memory_delete', {
        request: {
          path: loadedPath,
          project_id: projectId,
        },
      })

      setIsDeleteDialogOpen(false)
      setSelectedPath(null)
      setLoadedPath(null)
      setLoadedContent('')
      setDraftContent('')
      await refreshTree()
    } catch (error) {
      setFileError(error instanceof Error ? error.message : 'Failed to delete memory file.')
    } finally {
      setIsDeleting(false)
    }
  }

  const handleDeleteDialogCancel = () => {
    if (!isDeleting) {
      setIsDeleteDialogOpen(false)
    }
  }

  const handleDeleteDialogOpenChange = (isOpen: boolean) => {
    if (!isDeleting) {
      setIsDeleteDialogOpen(isOpen)
    }
  }

  return (
    <>
      <div className="w-full rounded-2xl border border-border/60 bg-background/70 shadow-xs">
        <div className="grid min-h-[680px] w-full grid-cols-[350px_minmax(0,1fr)]">
          <aside className="flex min-h-0 flex-col border-b border-border/60 bg-accent/10 xl:border-r xl:border-b-0">
            <div className="border-b border-border/60 px-4 py-4">
              <div className="flex items-center gap-2 text-sm font-medium text-foreground/90">
                <FolderOpen className="size-4 text-muted-foreground" />
                Memory browser
              </div>
            </div>

            <ScrollArea className="min-h-0 flex-1 overflow-y-auto h-[calc(100vh-11rem)]">
              <div className="p-3 pr-4">
                {isLoadingTree ? (
                  <div className="space-y-2 px-1 py-1">
                    {Array.from({ length: 5 }).map((_, index) => (
                      <div
                        key={`memory-tree-skeleton-${index}`}
                        className="h-8 animate-pulse rounded-md border border-border/50 bg-background/60"
                      />
                    ))}
                  </div>
                ) : treeHasNodes ? (
                  <TreeProvider
                    key={projectId}
                    showIcons
                    animateExpand={false}
                    className="w-full"
                    defaultExpandedIds={defaultExpandedIds}
                    indent={16}
                    selectedIds={selectedPath ? [`memory-file-${selectedPath}`] : []}
                    showLines={false}
                  >
                    <TreeView className="space-y-1">
                      <MemoryTreeBranch
                        level={0}
                        nodes={treeResult?.nodes ?? []}
                        selectedPath={selectedPath}
                        onSelectPath={handleSelectPath}
                      />
                    </TreeView>
                  </TreeProvider>
                ) : (
                  <div className="px-1 py-1">
                    <StatePanel
                      description="Managed project memories will appear here when files exist for this project."
                      icon={<FolderOpen className="size-4" />}
                      title="No memory files"
                    />
                  </div>
                )}
              </div>
            </ScrollArea>
          </aside>

          <section className="flex min-h-0 flex-col bg-background/40 h-[calc(100vh-7rem)]">
            <div className="border-b border-border/60 px-5 py-4">
              <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
                <div className="min-w-0 space-y-1">
                  <div className="flex items-center gap-2">
                    <div className="size-2 rounded-full bg-amber-400" />
                    <div className="truncate text-sm font-medium text-foreground/90">{headerFileName}</div>
                  </div>
                  <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-muted-foreground">
                    <div className="flex items-center gap-1.5">
                      <CalendarDays className="size-3.5" />
                      {headerPathLabel}
                    </div>
                    <div className="flex items-center gap-1.5">
                      <Clock3 className="size-3.5" />
                      {headerStatusLabel}
                    </div>
                  </div>
                </div>

                <div className="flex shrink-0 items-center gap-2">
                  <Button disabled={!canSave} size="sm" variant="outline" onClick={() => void handleSave()}>
                    {isSaving ? <LoaderCircle className="size-4 animate-spin" /> : <Save className="size-4" />}
                    {isSaving ? 'Saving...' : isDirty ? 'Save changes' : 'Saved'}
                  </Button>
                  <Button
                    disabled={!canDelete}
                    size="sm"
                    variant="destructive"
                    onClick={() => setIsDeleteDialogOpen(true)}
                  >
                    {isDeleting ? <LoaderCircle className="size-4 animate-spin" /> : <Trash2 className="size-4" />}
                    {isDeleting ? 'Deleting...' : 'Delete'}
                  </Button>
                </div>
              </div>
            </div>

            <div className="border-b border-border/60 px-5 py-3 text-xs text-muted-foreground">
              {fileError ? (
                <div className="flex items-start gap-2 rounded-lg border border-destructive/30 bg-destructive/5 px-3 py-2 text-destructive">
                  <AlertCircle className="mt-0.5 size-3.5 shrink-0" />
                  <span>{fileError}</span>
                </div>
              ) : selectedPath ? (
                isLoadingFile ? (
                  <span>Loading {selectedPath}.</span>
                ) : isSaving ? (
                  <span>Saving {loadedPath ?? selectedPath}.</span>
                ) : isDeleting ? (
                  <span>Deleting {loadedPath ?? selectedPath}.</span>
                ) : isDirty && loadedPath ? (
                  <span>Unsaved changes in {loadedPath}.</span>
                ) : loadedPath ? (
                  <span>Saved {loadedPath}.</span>
                ) : (
                  <span>Select a memory file from the tree to load its content.</span>
                )
              ) : (
                'Select a memory file from the tree to load its content.'
              )}
            </div>

            <div className="min-h-0 flex-1 p-2 relative h-full">
              <div className={cn('h-full', showEditorOverlay && 'pointer-events-none opacity-45')}>
                <MarkdownEditor
                  className="h-full"
                  placeholder={isLoadingFile ? 'Loading memory content…' : 'Select a memory file to view its content.'}
                  value={draftContent}
                  onChange={setDraftContent}
                />
              </div>

              {showEditorOverlay ? (
                <div className="absolute inset-3">
                  {fileError ? (
                    <StatePanel
                      description={fileError}
                      icon={<AlertCircle className="size-4" />}
                      title="Unable to load memory file"
                      tone="error"
                    />
                  ) : isDeleting ? (
                    <StatePanel
                      description={loadedPath ? `Removing ${loadedPath}.` : 'Removing selected memory file.'}
                      icon={<LoaderCircle className="size-4 animate-spin" />}
                      title="Deleting memory file"
                    />
                  ) : isLoadingFile ? (
                    <StatePanel
                      description={selectedPath ? `Loading ${selectedPath}.` : 'Loading selected memory file.'}
                      icon={<LoaderCircle className="size-4 animate-spin" />}
                      title="Loading memory file"
                    />
                  ) : (
                    <StatePanel
                      icon={<FileText className="size-4" />}
                      title={treeHasNodes ? 'No file selected' : 'Editor waiting for files'}
                      description={
                        treeHasNodes
                          ? 'Choose a file from the left rail to view and edit its markdown content.'
                          : 'This project has no memory files to open yet.'
                      }
                    />
                  )}
                </div>
              ) : null}
            </div>
          </section>
        </div>
      </div>

      <DeleteConfirmDialog
        description={deleteDescription}
        isOpen={isDeleteDialogOpen}
        title="Delete Memory File"
        onCancel={handleDeleteDialogCancel}
        onConfirm={handleDelete}
        onOpenChange={handleDeleteDialogOpenChange}
      />
      <Dialog open={isSwitchDialogOpen} onOpenChange={handleSwitchDialogOpenChange}>
        <DialogContent className="max-w-md" showCloseButton={false} size="xs">
          <DialogHeader>
            <DialogTitle>Unsaved changes</DialogTitle>
            <DialogDescription>
              {loadedPath && pendingSwitchFileName
                ? `Save changes to ${loadedPath} before opening ${pendingSwitchFileName}?`
                : 'Save changes before switching files?'}
            </DialogDescription>
          </DialogHeader>

          <DialogFooter>
            <Button disabled={isSaving} size="sm" variant="ghost" onClick={handleSwitchDialogCancel}>
              Cancel
            </Button>
            <Button disabled={isSaving} size="sm" variant="outline" onClick={handleSwitchDialogDiscard}>
              Discard
            </Button>
            <Button
              disabled={!canSave || isSaving}
              size="sm"
              variant="outline"
              onClick={() => void handleSwitchDialogSave()}
            >
              {isSaving ? 'Saving...' : 'Save'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  )
}
