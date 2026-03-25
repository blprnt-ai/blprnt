import { createFileRoute, Link } from '@tanstack/react-router'

const Index = () => {
  return (
    <div className="p-2">
      <h3>Welcome Home!</h3>
      <Link to="/onboarding">Onboarding</Link>
    </div>
  )
}

export const Route = createFileRoute('/')({
  component: Index,
})
