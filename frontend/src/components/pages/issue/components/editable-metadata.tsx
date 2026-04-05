import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'

export const EditableMetadata = ({
  label,
  onValueChange,
  options,
  placeholder,
  value,
}: {
  label: string
  onValueChange: (value: string) => void
  options: { label: string; value: string }[]
  placeholder?: string
  value: string
}) => {
  const selectedLabel = options.find((option) => option.value === value)?.label

  return (
    <div className="flex items-start">
      <div className="min-w-0 flex-1">
        <div className="text-xs uppercase tracking-[0.18em] text-muted-foreground/50">{label}</div>
        <Select
          value={value}
          onValueChange={(nextValue) => {
            onValueChange(nextValue ?? '')
          }}
        >
          <SelectTrigger className="w-full border-none text-muted-foreground/90 bg-transparent! pl-0" size="sm">
            <SelectValue placeholder={placeholder}>{selectedLabel}</SelectValue>
          </SelectTrigger>
          <SelectContent>
            {options.map((option) => (
              <SelectItem key={option.value} value={option.value}>
                {option.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
    </div>
  )
}
