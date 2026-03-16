interface ResultsCountProps {
  filtered: number
  total: number
}

export const ResultsCount = ({ filtered, total }: ResultsCountProps) => (
  <div className="text-xs text-muted-foreground">
    Showing {filtered} of {total} models
  </div>
)