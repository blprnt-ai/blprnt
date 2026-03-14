import { Loader2Icon } from 'lucide-react'
import { useEffect } from 'react'
import { InfoBox } from '@/components/atoms/boxes'
import { Button } from '@/components/atoms/button'
import { Separator } from '@/components/atoms/separator'
import { Section } from '@/components/organisms/page/section'
import { SectionField } from '@/components/organisms/page/section-field'
import { useAppViewModel } from '@/hooks/use-app-viewmodel'
import { useProvidersPageViewmodel } from './providers-page-viewmodel'

export const FnfProviders = () => {
  const { refreshProviders } = useAppViewModel()

  // biome-ignore lint/correctness/useExhaustiveDependencies: run once
  useEffect(() => {
    void refreshProviders()
  }, [])

  return (
    <Section>
      <SectionField
        title={
          <div className="flex flex-col gap-2">
            <div>Subscription Providers</div>
            <div className="text-muted-foreground text-sm font-light">
              Link your OpenAI/Codex and Anthropic/Claude subscription accounts.
            </div>
          </div>
        }
      >
        <div className="flex flex-col gap-2">
          <div className="flex items-center justify-between">
            <div>
              <div className="text-lg font-medium mb-2">OpenAI/Codex</div>
              <div className="text-muted-foreground text-sm font-light mb-4">
                You need to have codex-cli installed and logged in to your Codex account for this to work.
              </div>
            </div>
            <CodexProviderButton />
          </div>

          <Separator className="my-2" />

          <div>
            <div className="flex items-center justify-between">
              <div>
                <div className="text-lg font-medium mb-2">Anthropic/Claude</div>
                <div className="text-muted-foreground text-sm font-light mb-4">
                  You need to have claude-code cli installed and logged in to your Claude account for this to work.
                </div>
              </div>
              <ClaudeProviderButton />
            </div>
          </div>

          <InfoBox>
            After linking, it is strongly recommended to log out of your codex-cli/claude-code account and log back in,
            this ensures that blprnt and codex-cli/claude-code are using separate access and refresh tokens.
          </InfoBox>
          {/* <WarningBox>
            Anthropic has been known to block or restrict accounts that are used with automation or automation tools,
            such as blprnt.
          </WarningBox> */}
        </div>
      </SectionField>
    </Section>
  )
}

const CodexProviderButton = () => {
  const viewmodel = useProvidersPageViewmodel()
  const { hasCodex, refreshProviders } = useAppViewModel()

  const handleLinkCodex = async () => {
    await viewmodel.linkCodexAccount()
    await refreshProviders()
  }

  const handleUnlinkCodex = async () => {
    await viewmodel.unlinkCodexAccount()
    await refreshProviders()
  }

  if (viewmodel.isCodexLinking) {
    return (
      <Button disabled size="sm" variant="outline">
        <Loader2Icon className="size-4 animate-spin" />
      </Button>
    )
  }

  if (hasCodex) {
    return (
      <Button size="sm" variant="outline" onClick={handleUnlinkCodex}>
        Unlink
      </Button>
    )
  }

  return (
    <Button size="sm" variant="outline" onClick={handleLinkCodex}>
      Link
    </Button>
  )
}

const ClaudeProviderButton = () => {
  const viewmodel = useProvidersPageViewmodel()
  const { hasClaude, refreshProviders } = useAppViewModel()

  const handleLinkClaude = async () => {
    await viewmodel.linkClaudeAccount()
    await refreshProviders()
  }

  const handleUnlinkClaude = async () => {
    await viewmodel.unlinkClaudeAccount()
    await refreshProviders()
  }

  if (viewmodel.isClaudeLinking) {
    return (
      <Button disabled size="sm" variant="outline">
        <Loader2Icon className="size-4 animate-spin" />
      </Button>
    )
  }

  if (hasClaude) {
    return (
      <Button size="sm" variant="outline" onClick={handleUnlinkClaude}>
        Unlink
      </Button>
    )
  }

  return (
    <Button size="sm" variant="outline" onClick={handleLinkClaude}>
      Link
    </Button>
  )
}
