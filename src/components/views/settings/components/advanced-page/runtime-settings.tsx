import { Button } from '@/components/atoms/button'
import { Field, FieldGroup, FieldLabel } from '@/components/atoms/field'
import { basicToast } from '@/components/atoms/toaster'
import { Section } from '@/components/organisms/page/section'
import { SectionField } from '@/components/organisms/page/section-field'
import { useAdvancedPageViewModel } from './adcanced-page-viewmodel'

const formatRuntimeStatus = ({
  detected_version: detectedVersion,
  error,
  state,
}: {
  detected_version: string | null
  error: string | null
  state: string
}) => {
  if (state === 'available') {
    return `Available (${detectedVersion ?? 'unknown version'})`
  }

  if (state === 'missing') {
    return 'Missing'
  }

  return `Unavailable (${error ?? 'invocation failed'})`
}

const formatActiveRuntime = (
  activeRuntime:
    | {
        kind: string
        source: string
        version: string
      }
    | null
    | undefined,
) => {
  if (!activeRuntime) {
    return 'Not detected'
  }

  const kind = activeRuntime.kind === 'bun' ? 'Bun' : activeRuntime.kind
  const source = activeRuntime.source === 'path' ? 'PATH' : 'managed install'
  return `${kind} from ${source} (${activeRuntime.version})`
}

const formatQmdReadiness = ({ detail, state }: { detail: string; state: string }) => {
  const label =
    state === 'ready'
      ? 'Ready'
      : state === 'runtime_missing'
        ? 'Runtime missing'
        : state === 'qmd_missing_from_path'
          ? 'QMD missing'
          : state === 'qmd_unavailable'
            ? 'QMD unavailable'
            : 'Unavailable'

  return `${label}. ${detail}`
}

export const RuntimeSettings = () => {
  const viewmodel = useAdvancedPageViewModel()
  const health = viewmodel.jsRuntimeHealth
  const showPathHelp = health?.recommended_action.type === 'add_to_path' && !!health.path_help_snip

  return (
    <Section>
      <SectionField
        title={
          <div className="flex flex-col gap-2">
            <div>JavaScript runtime</div>
            <div className="text-muted-foreground text-sm font-light">
              blprnt uses a JavaScript runtime for memory indexing and related workflows.
            </div>
            <div className="text-muted-foreground text-sm font-light">
              This panel checks the active runtime, managed install, and QMD readiness across supported platforms.
            </div>
          </div>
        }
      >
        <FieldGroup>
          <Field orientation="horizontal">
            <div className="flex-1">
              <FieldLabel>Active runtime</FieldLabel>
              <div className="text-muted-foreground text-[13px] font-light">
                {viewmodel.jsRuntimeLoading ? 'Checking...' : formatActiveRuntime(health?.active_runtime)}
              </div>
            </div>
            <Button size="sm" variant="outline" onClick={() => viewmodel.loadJsRuntimeHealth()}>
              Refresh
            </Button>
          </Field>

          <Field orientation="horizontal">
            <div className="flex-1">
              <FieldLabel>Runtime on PATH</FieldLabel>
              <div className="text-muted-foreground text-[13px] font-light">
                {health ? formatRuntimeStatus(health.runtime_on_path) : 'Checking...'}
              </div>
            </div>
          </Field>

          <Field orientation="horizontal">
            <div className="flex-1">
              <FieldLabel>Managed runtime</FieldLabel>
              <div className="text-muted-foreground text-[13px] font-light">
                {health ? formatRuntimeStatus(health.managed_runtime) : 'Checking...'}
              </div>
            </div>
            {health?.install_supported && (
              <>
                <Button
                  disabled={viewmodel.jsRuntimeInstalling}
                  size="sm"
                  variant="outline"
                  onClick={() => viewmodel.installManagedJsRuntime(false)}
                >
                  Install
                </Button>
                <Button
                  disabled={viewmodel.jsRuntimeInstalling}
                  size="sm"
                  variant="destructive"
                  onClick={() => viewmodel.installManagedJsRuntime(true)}
                >
                  Reinstall
                </Button>
              </>
            )}
          </Field>

          <Field orientation="horizontal">
            <div className="flex-1">
              <FieldLabel>Managed install location</FieldLabel>
              <div className="text-muted-foreground text-[13px] font-light font-mono break-all">
                {health?.managed_runtime_path ?? 'Checking...'}
              </div>
            </div>
          </Field>

          <Field orientation="horizontal">
            <div className="flex-1">
              <FieldLabel>QMD readiness</FieldLabel>
              <div className="text-muted-foreground text-[13px] font-light">
                {health ? formatQmdReadiness(health.qmd_readiness) : 'Checking...'}
              </div>
            </div>
          </Field>

          <Field orientation="horizontal">
            <div className="flex-1">
              <FieldLabel>Recommended next action</FieldLabel>
              <div className="text-muted-foreground text-[13px] font-light">
                {health?.recommended_action.detail ?? 'Checking...'}
              </div>
            </div>
          </Field>

          {showPathHelp && (
            <Field orientation="horizontal">
              <div className="flex-1">
                <FieldLabel>PATH help</FieldLabel>
                <div className="text-muted-foreground text-[13px] font-light font-mono">{health.path_help_snip}</div>
              </div>
              <Button
                size="sm"
                variant="outline"
                onClick={async () => {
                  try {
                    await navigator.clipboard.writeText(health.path_help_snip!)
                    basicToast.success({ title: 'Copied to clipboard' })
                  } catch {
                    basicToast.error({ title: 'Failed to copy to clipboard' })
                  }
                }}
              >
                Copy
              </Button>
            </Field>
          )}
        </FieldGroup>
      </SectionField>
    </Section>
  )
}
