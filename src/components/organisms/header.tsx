import { ThemeToggle } from '../molecules/theme-toggle'
import { SidebarTrigger } from '../ui/sidebar'
import { HeaderBreadcrumbs } from './header-breadcrumbs'

export const Header = () => {
  return (
    <header className="mt-2 mb-px mx-1 flex h-12 min-w-0 items-center justify-between rounded-md bg-card px-2 py-2 ring-1 ring-sidebar-border">
      <div className="flex min-w-0 flex-1 items-center gap-2">
        <nav>
          <SidebarTrigger />
        </nav>
        <HeaderBreadcrumbs />
      </div>
      <div className="shrink-0">
        <ThemeToggle />
      </div>
    </header>
  )
}
