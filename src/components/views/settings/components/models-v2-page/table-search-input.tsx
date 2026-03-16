import { Input } from '@/components/atoms/input'

interface TableSearchInputProps {
  value: string
  onChange: (value: string) => void
  placeholder: string
}

export const TableSearchInput = ({ value, onChange, placeholder }: TableSearchInputProps) => (
  <Input
    className="max-w-md"
    placeholder={placeholder}
    value={value}
    onChange={(event) => onChange(event.target.value)}
  />
)