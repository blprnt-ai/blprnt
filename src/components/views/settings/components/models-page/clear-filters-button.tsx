import { X } from 'lucide-react'

interface ClearFiltersButtonProps {
  onClick: () => void
}

export const ClearFiltersButton = ({ onClick }: ClearFiltersButtonProps) => {
  return (
    <button
      className="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors h-9"
      type="button"
      onClick={onClick}
    >
      <X className="size-3" />
      Clear filters
    </button>
  )
}
