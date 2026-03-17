import { createFileRoute } from '@tanstack/react-router'

const Index = () => {
  return <div className="p-2"></div>
}

export const Route = createFileRoute('/')({
  component: Index,
})
