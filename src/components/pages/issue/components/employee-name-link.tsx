import { Link } from '@tanstack/react-router'
import { resolveEmployeeName } from '../utils'

interface EmployeeNameLinkProps {
  employeeId: string | null | undefined
  fallback: string
  className?: string
}

export const EmployeeNameLink = ({ employeeId, fallback, className }: EmployeeNameLinkProps) => {
  const label = resolveEmployeeName(employeeId, fallback)

  if (!employeeId) {
    return <span className={className}>{label}</span>
  }

  return (
    <Link className={className} params={{ employeeId }} title={label} to="/employees/$employeeId">
      {label}
    </Link>
  )
}