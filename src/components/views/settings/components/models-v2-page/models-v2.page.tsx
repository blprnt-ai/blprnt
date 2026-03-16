import { observer } from 'mobx-react-lite'
import { Button } from '@/components/atoms/button'
import { Table, TableBody, TableHead, TableHeader, TableRow } from '@/components/atoms/table'
import { Section } from '@/components/organisms/page/section'
import { SectionField } from '@/components/organisms/page/section-field'
import { ProviderMultiSelect } from '@/components/views/settings/components/models-page/provider-multi-select'
import { CustomModelForm } from './custom-model-form'
import { EmptyRow } from './empty-row'
import { ImportedModelRow } from './imported-model-row'
import { ModelsTableSection } from './models-table-section'
import { useModelsV2ViewModel } from './models-v2.viewmodel'
import { OpenRouterModelRow } from './open-router-model-row'
import { ResultsCount } from './results-count'
import { SortableHeader } from './sortable-header'
import { TableSearchInput } from './table-search-input'

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
                <Button
                  size="sm"
                  variant="outline"
                  onClick={
                    viewmodel.isCustomModelFormOpen ? viewmodel.closeCustomModelForm : viewmodel.openCustomModelForm
                  }
                >
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
                    Provider Model ID
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
                  <TableHead className="w-28 select-none hover:bg-muted/50 transition-colors">
                    <div className="flex items-center gap-1.5">Import</div>
                  </TableHead>
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
