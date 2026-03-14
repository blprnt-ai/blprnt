interface SectionButtonProps {
  label: React.ReactNode
  onClick?: () => void
}
export const SectionButton = ({ label, onClick = () => {} }: SectionButtonProps) => {
  return (
    <button
      className="text-primary hover:bg-gray-800/40 tracking-tight px-3.5 py-1.25 rounded-sm transition-colors duration-300 cursor-pointer"
      onClick={onClick}
    >
      {label}
    </button>
  )
}
