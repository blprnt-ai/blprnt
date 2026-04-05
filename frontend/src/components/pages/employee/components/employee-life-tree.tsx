import { ChevronRight, FileText, FolderTree, Pencil } from 'lucide-react'
import type { EmployeeLifeTreeNode } from '@/bindings/EmployeeLifeTreeNode'
import { cn } from '@/lib/utils'

interface EmployeeLifeTreeProps {
  nodes: EmployeeLifeTreeNode[]
  selectedPath: string | null
  onSelect: (path: string) => void
}

export const EmployeeLifeTree = ({ nodes, selectedPath, onSelect }: EmployeeLifeTreeProps) => (
  <div className="space-y-1">
    {nodes.map((node) => (
      <EmployeeLifeTreeNodeRow key={node.path} node={node} selectedPath={selectedPath} depth={0} onSelect={onSelect} />
    ))}
  </div>
)

interface EmployeeLifeTreeNodeRowProps {
  depth: number
  node: EmployeeLifeTreeNode
  selectedPath: string | null
  onSelect: (path: string) => void
}

const EmployeeLifeTreeNodeRow = ({ depth, node, selectedPath, onSelect }: EmployeeLifeTreeNodeRowProps) => {
  if (node.type === 'directory') {
    return (
      <div className="space-y-1">
        <div className="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm font-medium text-foreground/70">
          <FolderTree className="size-4" />
          <span style={{ paddingLeft: depth * 12 }}>{node.name}</span>
        </div>
        <div className="space-y-1">
          {node.children.map((child) => (
            <EmployeeLifeTreeNodeRow
              key={child.path}
              node={child}
              selectedPath={selectedPath}
              depth={depth + 1}
              onSelect={onSelect}
            />
          ))}
        </div>
      </div>
    )
  }

  return (
    <button
      className={cn(
        'flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm transition-colors',
        selectedPath === node.path ? 'bg-accent text-accent-foreground' : 'text-muted-foreground hover:bg-accent/50',
      )}
      style={{ paddingLeft: depth * 12 + 8 }}
      type="button"
      onClick={() => onSelect(node.path)}
    >
      <ChevronRight className={cn('size-4 opacity-0', selectedPath === node.path && 'opacity-100')} />
      <FileText className="size-4 shrink-0" />
      <span className="min-w-0 flex-1 truncate">{node.name}</span>
      {node.editable ? <Pencil className="size-3.5 shrink-0" /> : null}
    </button>
  )
}
