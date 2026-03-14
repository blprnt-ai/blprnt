import { Button } from '@/components/atoms/button'
import { Field, FieldGroup, FieldLabel } from '@/components/atoms/field'
import { basicToast } from '@/components/atoms/toaster'
import { Section } from '@/components/organisms/page/section'
import { SectionField } from '@/components/organisms/page/section-field'
import { isMac, useAdvancedPageViewModel } from './adcanced-page-viewmodel'

export const BunSettings = () => {
  const viewmodel = useAdvancedPageViewModel()

  return (
    <>
      {isMac && (
        <Section>
          <SectionField
            title={
              <div className="flex flex-col gap-2">
                <div>Bun (JavaScript runtime)</div>
                <div className="text-muted-foreground text-sm font-light">
                  blprnt can use Bun as a JavaScript runtime for some workflows.
                </div>
                <div className="text-muted-foreground text-sm font-light">
                  blprnt uses Bun from PATH. Installing to <span className="font-mono">~/.local/bin</span> may require
                  updating PATH and relaunching blprnt.
                </div>
              </div>
            }
          >
            <FieldGroup>
              <Field orientation="horizontal">
                <div className="flex-1">
                  <FieldLabel>Bun on PATH</FieldLabel>
                  <div className="text-muted-foreground text-[13px] font-light">
                    {viewmodel.bunLoading || !viewmodel.bunStatus
                      ? 'Checking...'
                      : viewmodel.bunStatus.bun.state === 'available'
                        ? `Available (${viewmodel.bunStatus.bun.detected_version ?? 'unknown version'})`
                        : viewmodel.bunStatus.bun.state === 'missing'
                          ? 'Missing'
                          : `Unavailable (${viewmodel.bunStatus.bun.error ?? 'invocation failed'})`}
                  </div>
                </div>
                <Button size="sm" variant="outline" onClick={() => viewmodel.loadBunStatus()}>
                  Refresh
                </Button>
              </Field>

              <Field orientation="horizontal">
                <div className="flex-1">
                  <FieldLabel>Install target</FieldLabel>
                  <div className="text-muted-foreground text-[13px] font-light font-mono">
                    {viewmodel.bunStatus?.install_target_path ?? '~/.local/bin/bun'}
                  </div>
                </div>
                <Button
                  disabled={viewmodel.bunInstalling}
                  size="sm"
                  variant="outline"
                  onClick={() => viewmodel.installBunUserLocal(false)}
                >
                  Install
                </Button>
                <Button
                  disabled={viewmodel.bunInstalling}
                  size="sm"
                  variant="destructive"
                  onClick={() => viewmodel.installBunUserLocal(true)}
                >
                  Overwrite
                </Button>
              </Field>

              <Field orientation="horizontal">
                <div className="flex-1">
                  <FieldLabel>PATH help</FieldLabel>
                  <div className="text-muted-foreground text-[13px] font-light font-mono">
                    export PATH=&quot;$HOME/.local/bin:$PATH&quot;
                  </div>
                </div>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={async () => {
                    try {
                      await navigator.clipboard.writeText('export PATH="$HOME/.local/bin:$PATH"')
                      basicToast.success({ title: 'Copied to clipboard' })
                    } catch {
                      basicToast.error({ title: 'Failed to copy to clipboard' })
                    }
                  }}
                >
                  Copy
                </Button>
              </Field>
            </FieldGroup>
          </SectionField>
        </Section>
      )}
    </>
  )
}
