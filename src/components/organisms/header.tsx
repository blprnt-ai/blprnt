import { ThemeToggle } from '../molecules/theme-toggle'
import { SidebarTrigger } from '../ui/sidebar'

export const Header = () => {
  return (
    <header className="bg-card rounded-md ring-1 ring-sidebar-border my-2 mx-1 px-2 py-2 flex items-center justify-between w-[calc(100%-0.75rem)]">
      <nav>
        <SidebarTrigger />
      </nav>
      <div>
        <ThemeToggle />
      </div>
    </header>
  )
}
