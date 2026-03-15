import { BotIcon, Bug, Building, Menu } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { Button } from '@/components/atoms/button'
import type { ReportBugDialogViewModel } from '@/components/dialogs/report-bug-dialog.viewmodel'
import { SidebarProvider } from '@/components/organisms/sidebar/sidebar.provider'
import { useSidebarViewmodel } from '@/components/organisms/sidebar/sidebar.viewmodel'
import { useAppViewModel } from '@/hooks/use-app-viewmodel'
import { cn } from '@/lib/utils/cn'
import { ProjectsTree } from './trees/projects-tree'

export const BlprntMenu = ({ reportBugViewmodel }: { reportBugViewmodel: ReportBugDialogViewModel }) => {
  return (
    <SidebarProvider reportBugViewmodel={reportBugViewmodel}>
      <BlprntMenuContent />
    </SidebarProvider>
  )
}

const BlprntMenuContent = observer(() => {
  const appViewmodel = useAppViewModel()
  const viewmodel = useSidebarViewmodel()

  const isReportBugAvailable = viewmodel.isReportBugAvailable

  return (
    <div
      className={cn(
        'flex flex-col justify-between h-full w-full bg-background rounded-lg border overflow-hidden pb-1 bg-gradient-glow-dark',
        !appViewmodel.isSidebarExpanded && 'w-10',
      )}
    >
      {/* <div className="h-full w-full absolute top-0 left-0 bg-grid-medium pointer-events-none" /> */}
      <div className="overflow-hidden" data-tour="sidebar-projects">
        <div
          className={cn(
            'pl-6.5 pr-2 border-b border-border h-8 mb-2',
            !appViewmodel.isSidebarExpanded && 'pr-0 pl-0 justify-center flex',
          )}
        >
          <div className="flex items-center justify-between text-sm h-full">
            <div className="text-primary font-semibold">
              <span
                className="font-mono tracking-widest whitespace-nowrap cursor-pointer"
                onClick={() => appViewmodel.toggleSidebarExpanded()}
              >
                {appViewmodel.isSidebarExpanded ? 'blprnt' : 'b'}
              </span>
            </div>
            {appViewmodel.isSidebarExpanded && (
              <Button size="icon" variant="link" onClick={() => appViewmodel.toggleSidebarExpanded()}>
                <Menu className="size-4" />
              </Button>
            )}
          </div>
        </div>

        {appViewmodel.isSidebarExpanded ? (
          <ProjectsTree />
        ) : (
          <Building className="size-4.5 text-center mx-auto text-muted-foreground" />
        )}
      </div>

      <div>
        <FooterItem
          className={cn('cursor-pointer text-muted-foreground hover:text-foreground transition-colors duration-300')}
          data-tour="user-account-models"
          role="button"
          onClick={() => viewmodel.openUserAccount('models')}
        >
          <BotIcon className="use-stroke-width" size={18} strokeWidth={1} />
          {appViewmodel.isSidebarExpanded ? ' Settings' : ''}
        </FooterItem>

        <FooterItem
          aria-disabled={!isReportBugAvailable}
          role="button"
          className={cn(
            'transition-colors duration-300',
            isReportBugAvailable
              ? 'cursor-pointer text-muted-foreground hover:text-foreground'
              : 'cursor-not-allowed text-muted-foreground/50',
          )}
          onClick={isReportBugAvailable ? viewmodel.openReportBug : undefined}
        >
          <Bug className="use-stroke-width" size={18} strokeWidth={1} />
          {appViewmodel.isSidebarExpanded ? ' Report Bug' : ''}
        </FooterItem>
      </div>
    </div>
  )
})

const FooterItem = ({ className, ...props }: React.ComponentProps<'div'>) => {
  const appViewmodel = useAppViewModel()
  return (
    <div
      className={cn(
        'flex items-center border-t border-border px-4 gap-2 text-sm h-8 bg-background hover:bg-background-2 transition-colors duration-300',
        !appViewmodel.isSidebarExpanded && 'justify-center px-0',
        className,
      )}
      {...props}
    />
  )
}
