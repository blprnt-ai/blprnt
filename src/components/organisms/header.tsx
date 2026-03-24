import { SidebarTrigger } from '../ui/sidebar'

export const Header = () => {
  return (
    <header className="bg-card border-b px-4 py-2 flex items-center justify-between w-full">
      <SidebarTrigger />
    </header>
  )
}
