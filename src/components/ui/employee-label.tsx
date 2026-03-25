import {
  Baby,
  Book,
  Bot,
  Brain,
  Briefcase,
  Building,
  Cat,
  Dog,
  FlaskConical,
  type LucideProps,
  ShieldUser,
  Smile,
  Stethoscope,
  User,
  Wallet,
} from 'lucide-react'
import { type ColorVariant, TextColoredSpan } from '@/components/ui/colors'

export type LucideIcon = React.ForwardRefExoticComponent<Omit<LucideProps, 'ref'> & React.RefAttributes<SVGSVGElement>>

export const employeeIcons: { icon: LucideIcon; name: string; value: string; default?: boolean }[] = [
  { icon: Baby, name: 'Baby', value: 'baby' },
  { icon: Book, name: 'Book', value: 'book' },
  { icon: Bot, name: 'Bot', value: 'bot' },
  { icon: Brain, name: 'Brain', value: 'brain' },
  { icon: Briefcase, name: 'Briefcase', value: 'briefcase' },
  { icon: Building, name: 'Building', value: 'building' },
  { icon: Cat, name: 'Cat', value: 'cat' },
  { icon: Dog, name: 'Dog', value: 'dog' },
  { icon: FlaskConical, name: 'FlaskConical', value: 'flask-conical' },
  { icon: ShieldUser, name: 'ShieldUser', value: 'shield-user' },
  { icon: Smile, name: 'Smile', value: 'smile' },
  { icon: Stethoscope, name: 'Stethoscope', value: 'stethoscope' },
  { default: true, icon: User, name: 'User', value: 'user' },
  { icon: Wallet, name: 'Wallet', value: 'wallet' },
]

export const employeeIconValueToIcon = (value: string) => {
  return employeeIcons.find((icon) => icon.value === value)?.icon
}

export const EmployeeLabel = ({
  name,
  Icon,
  color,
}: {
  name: string
  Icon: React.ComponentType<{ className: string }>
  color: ColorVariant
}) => {
  return (
    <div className="flex items-center gap-2">
      <TextColoredSpan color={color}>
        <Icon className="size-4" />
      </TextColoredSpan>
      <span>{name}</span>
    </div>
  )
}
