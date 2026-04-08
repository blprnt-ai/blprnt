import { BotIcon, MessageCircleIcon, ServerIcon } from 'lucide-react'
import { Page } from '@/components/layouts/page'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { McpSettingsSection } from './components/mcp-settings-section'
import { ProvidersSettingsSection } from './components/providers-settings-section'
import { TelegramSettingsSection } from './components/telegram-settings-section'

export const SettingsPage = () => {
  return (
    <Page className="overflow-y-auto px-3 pb-6 md:px-5">
      <div className="mx-auto flex w-full max-w-7xl flex-col gap-4">
        <div className="space-y-1">
          <h1 className="text-2xl font-semibold tracking-tight">Settings</h1>
        </div>

        <Tabs defaultValue="providers">
          <TabsList variant="line">
            <TabsTrigger value="mcp">
              <ServerIcon className="size-4" />
              MCP
            </TabsTrigger>
            <TabsTrigger value="providers">
              <BotIcon className="size-4" />
              Providers
            </TabsTrigger>
            <TabsTrigger value="telegram">
              <MessageCircleIcon className="size-4" />
              Telegram
            </TabsTrigger>
          </TabsList>

          <TabsContent className="mt-5" value="mcp">
            <McpSettingsSection />
          </TabsContent>

          <TabsContent className="mt-5" value="providers">
            <ProvidersSettingsSection />
          </TabsContent>

          <TabsContent className="mt-5" value="telegram">
            <TelegramSettingsSection />
          </TabsContent>
        </Tabs>
      </div>
    </Page>
  )
}
