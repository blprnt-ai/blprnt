import type { ReactNode } from 'react'
import { Page } from '@/components/layouts/page'
import { ThemeToggle } from '@/components/molecules/theme-toggle'

interface AuthShellProps {
  children: ReactNode
}

export const AuthShell = ({ children }: AuthShellProps) => {
  return (
    <Page className="h-screen max-h-screen pt-0!">
      <div className="relative flex h-screen w-screen items-center justify-center px-4">
        <div className="absolute top-2 right-2 opacity-30 transition-opacity duration-300 hover:opacity-100">
          <ThemeToggle />
        </div>
        <div className="w-full max-w-md">{children}</div>
      </div>
    </Page>
  )
}
