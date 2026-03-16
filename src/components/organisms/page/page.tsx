import { useEffect, useState } from 'react'
import { Separator } from '@/components/atoms/separator'
import { cn } from '@/lib/utils/cn'
import { PageTitle } from './page-title'

export interface Tab {
  icon: React.ReactNode
  label: string
  path: string
  title: string | null
  content: React.ReactNode
}

export interface Action {
  icon: React.ReactNode
  label: React.ReactNode
  enabled: boolean
  variant: 'primary' | 'secondary' | 'success' | 'danger'
  onClick: () => void
}

interface PageProps {
  title: string
  subtitle: string
  tabs: Tab[]
  actions?: Action[]
  activeTabPath?: string
  initialTab?: string
  onTabChange?: (path: string) => void
}

export const Page = ({ title, subtitle, tabs, actions, activeTabPath, initialTab, onTabChange }: PageProps) => {
  if (tabs.length === 0) throw new Error('Tabs are required')
  const [activeTabPathInternal, setActiveTabPathInternal] = useState<string>(tabs[0].path)
  const activeTab = tabs.find((tab) => tab.path === activeTabPathInternal)

  useEffect(() => {
    if (activeTabPath && activeTabPath !== activeTabPathInternal) setActiveTabPathInternal(activeTabPath)
  }, [activeTabPath, activeTabPathInternal])

  useEffect(() => {
    if (initialTab) setActiveTabPathInternal(initialTab)
  }, [initialTab])

  const handleTabChange = (path: string) => {
    setActiveTabPathInternal(path)
    onTabChange?.(path)
  }

  return (
    <div className="flex gap-2 h-full w-full p-2">
      <div className="h-full border rounded-md pt-6 px-3 w-57 shrink-0 bg-accent">
        <div className="flex flex-col h-full gap-6">
          <div>
            <div className="px-3">
              <PageTitle title={title} />
              <div className="text-sm text-muted-foreground tracking-tight font-light mb-6">{subtitle}</div>
            </div>
            <div className="flex flex-col gap-0.5">
              {tabs.map((tab) => (
                <NavbarButton
                  key={tab.label}
                  icon={tab.icon}
                  isActive={activeTabPathInternal === tab.path}
                  label={tab.label}
                  onClick={() => handleTabChange(tab.path)}
                />
              ))}
            </div>
          </div>

          {actions && actions.length > 0 && (
            <>
              <Separator />
              <div className="flex flex-col gap-1">
                {actions?.map((action, index) => (
                  <ActionButton key={index} action={action} />
                ))}
              </div>
            </>
          )}
        </div>
      </div>
      <div className="h-full bg-gradient-glow border rounded-md pt-6 px-8 w-full overflow-y-auto">
        {activeTab?.title && <PageTitle title={activeTab?.title} />}
        {activeTab?.content}
      </div>
    </div>
  )
}

interface NavbarButtonProps {
  icon: React.ReactNode
  label: string
  isActive: boolean
  onClick: () => void
}

const NavbarButton = ({ icon, label, isActive, onClick }: NavbarButtonProps) => {
  return (
    <button
      className={cn(
        'flex items-center gap-3.25 w-full justify-start px-3 py-1.5 rounded-md',
        'text-muted-foreground hover:bg-accent/60',
        'transition-colors duration-300 cursor-pointer',
        'text-sm font-medium tracking-normal',
        '[&_svg]:size-4',
        isActive && 'text-primary bg-linear-to-r from-transparent to-primary/30',
      )}
      onClick={onClick}
    >
      {icon}
      <div>{label}</div>
    </button>
  )
}

interface ActionButtonProps {
  action: Action
}

const ActionButton = ({ action }: ActionButtonProps) => {
  const { icon, label, onClick, variant, enabled } = action

  return (
    <button
      className={cn(
        'flex items-center gap-3.25 w-full justify-start px-3 py-1.5 rounded-md',

        !enabled && 'border cursor-not-allowed text-muted-foreground/70',
        enabled && 'cursor-pointer',

        enabled && variant === 'primary' && 'bg-primary/30 hover:text-muted hover:bg-primary',
        enabled && variant === 'secondary' && 'bg-accent/30 hover:bg-accent',
        enabled && variant === 'success' && 'bg-success/30 hover:text-muted hover:bg-success',
        enabled && variant === 'danger' && 'bg-destructive/30 hover:text-background hover:bg-destructive',

        !enabled && variant === 'primary' && 'border-primary/20',
        !enabled && variant === 'secondary' && 'border-secondary',
        !enabled && variant === 'success' && 'border-success/20',
        !enabled && variant === 'danger' && 'border-destructive/20',

        'transition-colors duration-300',
        'text-sm font-medium tracking-normal',
        '[&_svg]:size-4',
      )}
      onClick={onClick}
    >
      {icon}
      <div className="w-full">{label}</div>
    </button>
  )
}
