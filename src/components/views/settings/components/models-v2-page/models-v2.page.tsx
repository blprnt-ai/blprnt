import { ArrowDown, ArrowUp, ArrowUpDown } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import type { ReactNode } from 'react'
import { Button } from '@/components/atoms/button'
import { Input } from '@/components/atoms/input'
import { Switch } from '@/components/atoms/switch'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/atoms/table'
import { Section } from '@/components/organisms/page/section'
import { SectionField } from '@/components/organisms/page/section-field'
import { cn } from '@/lib/utils/cn'
import { ModelUsageCell } from '../models-page/model-usage-cell'
import { ProviderMultiSelect } from '../models-page/provider-multi-select'
import { type BlprntModel, type OpenRouterModel, type SortState, useModelsV2ViewModel } from './models-v2.viewmodel'

interface SortableHeaderProps<TColumn extends string> {
  column: TColumn
  sort: SortState<TColumn>
  onSort: (column: TColumn) => void
  className?: string
  children: ReactNode
}

const formatContextLength = (value: number) => {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`
  if (value >= 1_000) return `${(value / 1_000).toFixed(0)}K`
  return value.toString()
}

const SortableHeader = <TColumn extends string>({
  column,
  sort,
  onSort,
  className,
  children,
}: SortableHeaderProps<TColumn>) => {
  const isActive = sort.column === column
  const Icon = !isActive ? ArrowUpDown : sort.direction === 'asc' ? ArrowUp : ArrowDown

  return (
    <TableHead
      className={cn('cursor-pointer select-none hover:bg-muted/50 transition-colors', className)}
      onClick={() => onSort(column)}
    >
      <div className="flex items-center gap-1.5">
        {children}
        <Icon className={cn('size-3.5', !isActive && 'text-muted-foreground/50')} />
      </div>
    </TableHead>
  )
}

export const ModelsV2Page = observer(() => {
  const viewmodel = useModelsV2ViewModel()

  return (
    <Section>
      <SectionField
        title={
          <div className="flex flex-col gap-2">
            <div>Models V2</div>
            <div className="text-muted-foreground text-sm font-light">
              Import OpenRouter models and tune the local catalog.
            </div>
          </div>
        }
      >
        <div className="w-full space-y-6">
          {viewmodel.isLoading && viewmodel.blprntModels.length === 0 && viewmodel.openRouterModels.length === 0 ? (
            <div className="text-sm text-muted-foreground">Loading models...</div>
          ) : null}

          <ModelsTableSection description="Imported models stored locally for blprnt." title="Imported Models">
            <div className="space-y-2">
              <div className="flex justify-start">
                <Button size="sm" variant="outline" onClick={viewmodel.isCustomModelFormOpen ? viewmodel.closeCustomModelForm : viewmodel.openCustomModelForm}>
                  {viewmodel.isCustomModelFormOpen ? 'Cancel custom model' : 'Add custom model'}
                </Button>
              </div>
              {viewmodel.isCustomModelFormOpen ? <CustomModelForm /> : null}
              <TableSearchInput
                placeholder="Search imported models..."
                value={viewmodel.importedSearchQuery}
                onChange={viewmodel.setImportedSearchQuery}
              />
              <ResultsCount filtered={viewmodel.sortedImportedModels.length} total={viewmodel.blprntModels.length} />
            </div>
            <Table className="w-full">
              <TableHeader>
                <TableRow>
                  <SortableHeader column="enabled" sort={viewmodel.importedSort} onSort={viewmodel.setImportedSort}>
                    Enabled
                  </SortableHeader>
                  <SortableHeader column="name" sort={viewmodel.importedSort} onSort={viewmodel.setImportedSort}>
                    Model
                  </SortableHeader>
                  <SortableHeader
                    column="provider_slug"
                    sort={viewmodel.importedSort}
                    onSort={viewmodel.setImportedSort}
                  >
                    Provider Slug
                  </SortableHeader>
                  <SortableHeader
                    column="context_length"
                    sort={viewmodel.importedSort}
                    onSort={viewmodel.setImportedSort}
                  >
                    Context
                  </SortableHeader>
                  <TableHead className="w-24">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {viewmodel.sortedImportedModels.length === 0 ? (
                  <EmptyRow
                    colSpan={5}
                    message={
                      viewmodel.importedSearchQuery.trim()
                        ? 'No imported models match your filters.'
                        : 'No imported models yet. Import something from OpenRouter below.'
                    }
                  />
                ) : (
                  viewmodel.sortedImportedModels.map((model) => (
                    <ImportedModelRow
                      key={model.id}
                      model={model}
                      onDelete={viewmodel.deleteModel}
                      onProviderSlugChange={viewmodel.setModelProviderSlug}
                      onToggle={viewmodel.toggleModel}
                    />
                  ))
                )}
              </TableBody>
            </Table>
          </ModelsTableSection>

          <ModelsTableSection description="Available models fetched from OpenRouter." title="OpenRouter Models">
            <div className="space-y-2">
              <div className="flex flex-wrap items-end gap-4">
                <TableSearchInput
                  placeholder="Search OpenRouter models..."
                  value={viewmodel.openRouterSearchQuery}
                  onChange={viewmodel.setOpenRouterSearchQuery}
                />
                <div className="flex flex-col gap-1.5">
                  <span className="text-xs text-muted-foreground">Provider</span>
                  <ProviderMultiSelect
                    providers={viewmodel.openRouterProviders}
                    selected={viewmodel.selectedOpenRouterProviders}
                    onToggle={viewmodel.toggleOpenRouterProvider}
                  />
                </div>
              </div>
              <ResultsCount
                filtered={viewmodel.sortedOpenRouterModels.length}
                total={viewmodel.openRouterModels.length}
              />
            </div>
            <Table className="w-full">
              <TableHeader>
                <TableRow>
                  <SortableHeader column="name" sort={viewmodel.openRouterSort} onSort={viewmodel.setOpenRouterSort}>
                    Model
                  </SortableHeader>
                  <SortableHeader column="usage" sort={viewmodel.openRouterSort} onSort={viewmodel.setOpenRouterSort}>
                    Usage
                  </SortableHeader>
                  <SortableHeader
                    column="context_length"
                    sort={viewmodel.openRouterSort}
                    onSort={viewmodel.setOpenRouterSort}
                  >
                    Context
                  </SortableHeader>
                  <SortableHeader
                    className="w-28"
                    column="imported"
                    sort={viewmodel.openRouterSort}
                    onSort={viewmodel.setOpenRouterSort}
                  >
                    Import
                  </SortableHeader>
                </TableRow>
              </TableHeader>
              <TableBody>
                {viewmodel.sortedOpenRouterModels.length === 0 ? (
                  <EmptyRow
                    colSpan={4}
                    message={
                      viewmodel.openRouterSearchQuery.trim() || viewmodel.selectedOpenRouterProviders.length > 0
                        ? 'No OpenRouter models match your filters.'
                        : 'No OpenRouter models loaded.'
                    }
                  />
                ) : (
                  viewmodel.sortedOpenRouterModels.map((model) => (
                    <OpenRouterModelRow
                      key={model.id}
                      imported={viewmodel.importedIds.has(model.id)}
                      maxPromptPrice={viewmodel.maxOpenRouterPromptPrice}
                      minPromptPrice={viewmodel.minOpenRouterPromptPrice}
                      model={model}
                      onImport={viewmodel.importModel}
                    />
                  ))
                )}
              </TableBody>
            </Table>
          </ModelsTableSection>
        </div>
      </SectionField>
    </Section>
  )
})

interface ModelsTableSectionProps {
  title: string
  description: string
  children: ReactNode
}

const ModelsTableSection = ({ title, description, children }: ModelsTableSectionProps) => {
  return (
    <div className="w-full space-y-2">
      <div className="space-y-1">
        <div className="text-sm font-semibold">{title}</div>
        <div className="text-muted-foreground text-xs font-light">{description}</div>
      </div>
      {children}
    </div>
  )
}

const TableSearchInput = ({
  value,
  onChange,
  placeholder,
}: {
  value: string
  onChange: (value: string) => void
  placeholder: string
}) => (
  <Input
    className="max-w-md"
    placeholder={placeholder}
    value={value}
    onChange={(event) => onChange(event.target.value)}
  />
)

const ResultsCount = ({ filtered, total }: { filtered: number; total: number }) => (
  <div className="text-xs text-muted-foreground">
    Showing {filtered} of {total} models
  </div>
)

const EmptyRow = ({ colSpan, message }: { colSpan: number; message: string }) => (
  <TableRow>
    <TableCell className="py-8 text-center text-muted-foreground" colSpan={colSpan}>
      {message}
    </TableCell>
  </TableRow>
)

const ImportedModelRow = ({
  model,
  onDelete,
  onToggle,
  onProviderSlugChange,
}: {
  model: BlprntModel
  onDelete: (model: BlprntModel) => void
  onToggle: (model: BlprntModel) => void
  onProviderSlugChange: (model: BlprntModel, providerSlug: string) => void
}) => {
  return (
    <TableRow>
      <TableCell>
        <Switch checked={model.enabled} onCheckedChange={() => void onToggle(model)} />
      </TableCell>
      <TableCell className="max-w-md truncate">{model.name}</TableCell>
      <TableCell>
        <Input
          className="h-8 min-w-48"
          value={model.provider_slug ?? ''}
          onChange={(event) => void onProviderSlugChange(model, event.target.value)}
        />
      </TableCell>
      <TableCell>{formatContextLength(model.context_length)}</TableCell>
      <TableCell>
        <Button size="sm" variant="ghost" onClick={() => void onDelete(model)}>
          Delete
        </Button>
      </TableCell>
    </TableRow>
  )
}

const CustomModelForm = observer(() => {
  const viewmodel = useModelsV2ViewModel()

  return (
    <div className="rounded-md border p-4 space-y-3">
      <div className="grid gap-3 md:grid-cols-2">
        <FormField
          label="Model ID"
          value={viewmodel.customModelDraft.id}
          onChange={(value) => viewmodel.setCustomModelDraftField('id', value)}
        />
        <FormField
          label="Display name"
          value={viewmodel.customModelDraft.name}
          onChange={(value) => viewmodel.setCustomModelDraftField('name', value)}
        />
        <FormField
          label="Context length"
          type="number"
          value={viewmodel.customModelDraft.contextLength}
          onChange={(value) => viewmodel.setCustomModelDraftField('contextLength', value)}
        />
        <FormField
          label="Provider slug"
          value={viewmodel.customModelDraft.providerSlug}
          onChange={(value) => viewmodel.setCustomModelDraftField('providerSlug', value)}
        />
        <FormField
          label="Prompt pricing"
          value={viewmodel.customModelDraft.promptPrice}
          onChange={(value) => viewmodel.setCustomModelDraftField('promptPrice', value)}
        />
        <FormField
          label="Completion pricing"
          value={viewmodel.customModelDraft.completionPrice}
          onChange={(value) => viewmodel.setCustomModelDraftField('completionPrice', value)}
        />
        <FormField
          label="Request pricing"
          value={viewmodel.customModelDraft.requestPrice}
          onChange={(value) => viewmodel.setCustomModelDraftField('requestPrice', value)}
        />
        <FormField
          label="Image pricing"
          value={viewmodel.customModelDraft.imagePrice}
          onChange={(value) => viewmodel.setCustomModelDraftField('imagePrice', value)}
        />
      </div>
      {viewmodel.customModelFormError ? (
        <div className="text-sm text-destructive">{viewmodel.customModelFormError}</div>
      ) : null}
      <div className="flex gap-2">
        <Button size="sm" onClick={() => void viewmodel.saveCustomModel()}>
          Save custom model
        </Button>
        <Button size="sm" variant="ghost" onClick={viewmodel.closeCustomModelForm}>
          Cancel
        </Button>
      </div>
    </div>
  )
})

const FormField = ({
  label,
  value,
  onChange,
  type = 'text',
}: {
  label: string
  value: string
  onChange: (value: string) => void
  type?: 'text' | 'number'
}) => (
  <label className="flex flex-col gap-1.5">
    <span className="text-xs text-muted-foreground">{label}</span>
    <Input type={type} value={value} onChange={(event) => onChange(event.target.value)} />
  </label>
)

const OpenRouterModelRow = ({
  model,
  imported,
  minPromptPrice,
  maxPromptPrice,
  onImport,
}: {
  model: OpenRouterModel
  imported: boolean
  minPromptPrice: number
  maxPromptPrice: number
  onImport: (model: OpenRouterModel) => void
}) => {
  return (
    <TableRow>
      <TableCell className="max-w-md truncate">{model.name}</TableCell>
      <TableCell>
        <ModelUsageCell
          maxOutputPrice={maxPromptPrice}
          minOutputPrice={minPromptPrice}
          outputPrice={model.pricing.prompt}
        />
      </TableCell>
      <TableCell>{formatContextLength(model.context_length)}</TableCell>
      <TableCell>
        <Button
          disabled={imported}
          size="sm"
          variant={imported ? 'secondary' : 'outline'}
          onClick={() => void onImport(model)}
        >
          {imported ? 'Imported' : 'Import'}
        </Button>
      </TableCell>
    </TableRow>
  )
}
