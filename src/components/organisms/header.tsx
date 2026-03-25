import { ThemeToggle } from '../molecules/theme-toggle'
import { SidebarTrigger } from '../ui/sidebar'

export const Header = () => {
  return (
    <header className="bg-card rounded-md ring-1 ring-sidebar-border m-2 px-4 py-2 flex items-center justify-between w-[calc(100%-1rem)]">
      <nav>
        <SidebarTrigger />
      </nav>
      <div>
        <ThemeToggle />
      </div>
    </header>
  )
}
