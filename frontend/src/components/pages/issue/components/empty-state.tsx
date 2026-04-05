export const EmptyState = ({ title, description }: { title: string; description: string }) => {
  return (
    <div className="rounded-sm border border-dashed border-border/70 p-6 text-center">
      <div className="font-medium">{title}</div>
      <p className="mt-1 text-sm text-muted-foreground">{description}</p>
    </div>
  )
}
