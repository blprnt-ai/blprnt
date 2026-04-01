export const IssueBadge = ({ children }: { children: React.ReactNode }) => {
  return (
    <span className="rounded-full border border-border/60 bg-muted/30 px-2.5 py-1 text-[11px] font-medium">
      {children}
    </span>
  )
}
