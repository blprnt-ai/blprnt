import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/issues/')({
  component: RouteComponent,
})

function RouteComponent() {
  return <div>Hello "/issues/"!</div>
}
