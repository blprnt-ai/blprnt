import { ChevronDown, ChevronRight, FileText, FolderTree } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { useState } from 'react'
import type { ProjectMemoryTreeNode } from '@/lib/api/projects'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { cn } from '@/lib/utils'
import { useProjectViewmodel } from '../project.viewmodel'

export const ProjectMemoryTree = observer(() => {
  const viewmodel = useProjectViewmodel()

  return (
    <Card className="border-border/60">
      <CardHeader>
        <CardTitle className="text-base">Files</CardTitle>
      </CardHeader>
      <CardContent className="space-y-1">
        {viewmodel.memoryTreeNodes.map((node) => (
          <MemoryNode key={node.path} node={node} />
        ))}
      </CardContent>
    </Card>
  )
})

const MemoryNode = observer(({ depth = 0, node }: { depth?: number; node: ProjectMemoryTreeNode }) => {
  const viewmodel = useProjectViewmodel()
  const [isOpen, setIsOpen] = useState(true)

  if (node.type === 'directory') {
    return (
      <div className="space-y-1">
        <button
          className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm hover:bg-muted/40"
          style={{ paddingLeft: `${depth * 16 + 8}px` }}
          type="button"
          onClick={() => setIsOpen((value) => !value)}
        >
          {isOpen ? <ChevronDown className="size-4 text-muted-foreground" /> : <ChevronRight className="size-4 text-muted-foreground" />}
          <FolderTree className="size-4 text-muted-foreground" />
          <span className="truncate">{node.name}</span>
        </button>

        {isOpen
          ? node.children.map((child) => <MemoryNode key={child.path} depth={depth + 1} node={child} />)
          : null}
      </div>
    )
  }

  const isSelected = viewmodel.selectedMemoryPath === node.path

  return (
    <button
      className={cn(
        'flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm hover:bg-muted/40',
        isSelected && 'bg-muted text-foreground',
      )}
      style={{ paddingLeft: `${depth * 16 + 28}px` }}
      type="button"
      onClick={() => void viewmodel.selectMemoryPath(node.path)}
    >
      <FileText className="size-4 text-muted-foreground" />
      <span className="truncate">{node.name}</span>
    </button>
  )
})