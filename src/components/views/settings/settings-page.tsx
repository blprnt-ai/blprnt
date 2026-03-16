import { Bot, Plug, Server, UserRound, Wrench } from 'lucide-react'
import { Page, type Tab } from '@/components/organisms/page/page'
import { PersonalitiesPage } from '@/components/views/personalities/personalities-page'
import { AdvancedPage } from './components/advanced-page'
import { IntegrationsPage } from './components/integrations-page'
import { McpSettingsPage } from './components/mcp-settings-page'
import { ModelsV2Page } from './components/models-v2-page'
import { ProvidersPage } from './components/providers-page/providers-page'

export type SettingsTabs = 'models' | 'mcp' | 'personalities' | 'advanced' | 'integrations' | 'providers'

interface SettingsPageProps {
  initialTab: SettingsTabs
}

export const SettingsPage = ({ initialTab }: SettingsPageProps) => {
  const tabs: Tab[] = [
    {
      content: <ModelsV2Page />,
      icon: <Bot className="size-4" />,
      label: 'Models',
      path: 'models',
      title: 'Models',
    },
    {
      content: <ProvidersPage />,
      icon: <Server className="size-4" />,
      label: 'Providers',
      path: 'providers',
      title: 'Providers',
    },
    {
      content: <PersonalitiesPage />,
      icon: <UserRound className="size-4" />,
      label: 'Personalities',
      path: 'personalities',
      title: 'Personalities',
    },
    {
      content: <McpSettingsPage />,
      icon: <Server className="size-4" />,
      label: 'MCP',
      path: 'mcp',
      title: 'MCP Servers',
    },
    {
      content: <IntegrationsPage />,
      icon: <Plug className="size-4" />,
      label: 'Integrations',
      path: 'integrations',
      title: 'Integrations',
    },
    {
      content: <AdvancedPage />,
      icon: <Wrench className="size-4" />,
      label: 'Advanced',
      path: 'advanced',
      title: 'Advanced',
    },
  ]

  return <Page initialTab={initialTab} subtitle="Manage your account and preferences." tabs={tabs} title="Settings" />
}
