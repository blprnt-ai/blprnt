export const MetadataRow = ({ label, value }: { label: string; value: string }) => {
  return (
    <div className="flex items-start gap-3">
      <div className="min-w-0">
        <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground/50">{label}</div>
        <div className="mt-1 wrap-break-word text-sm font-medium text-muted-foreground/90">{value}</div>
      </div>
    </div>
  )
}
