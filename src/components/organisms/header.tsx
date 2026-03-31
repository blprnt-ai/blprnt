import { ThemeToggle } from '../molecules/theme-toggle'
import { SidebarTrigger } from '../ui/sidebar'
import { HeaderBreadcrumbs } from './header-breadcrumbs'

export const Header = () => {
  return (
    <header className="bg-card rounded-md ring-1 ring-sidebar-border mt-2 mb-px mx-1 px-2 py-2 flex items-center justify-between w-[calc(100%-0.75rem)] h-12">
      <div className="flex min-w-0 items-center gap-2">
        <nav>
          <SidebarTrigger />
        </nav>
        <HeaderBreadcrumbs />
      </div>
      <div>
        <ThemeToggle />
      </div>
    </header>
  )
}
