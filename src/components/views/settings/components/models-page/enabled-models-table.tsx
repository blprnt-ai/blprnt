import { useMemo, useState } from 'react'
import { Table, TableBody, TableCell, TableRow } from '@/components/atoms/table'
import { Section } from '@/components/organisms/page/section'
import { SectionField } from '@/components/organisms/page/section-field'
import type { ModelCatalogItem } from '@/lib/models/app.model'
import { AutoRouterFilter } from './auto-router-filter'
import { ClearFiltersButton } from './clear-filters-button'
import { ModelRow } from './model-row'
import { ModelsTableHeader } from './models-table-header'
import { OauthFilter } from './oauth-filter'
import { ProviderMultiSelect } from './provider-multi-select'
import { ReasoningFilter } from './reasoning-filter'
import { ResultsCount } from './results-count'
import { SearchInput } from './search-input'
import { type EnabledFilterValue, StatusFilter } from './status-filter'
import type { SortColumn, SortState } from './types'
import { UsageFilterSelect, type UsageFilterValue } from './usage-filter-select'
import { fuzzyMatch, getLabelForValue } from './utils'

interface EnabledModelsTableProps {
  llmModels: ModelCatalogItem[]
  toggleSlug: (slug: string) => void
}

export const EnabledModelsTable = ({ llmModels, toggleSlug }: EnabledModelsTableProps) => {
  const [sort, setSort] = useState<SortState>({ column: 'enabled', direction: 'desc' })

  const [searchQuery, setSearchQuery] = useState('')
  const [enabledFilter, setEnabledFilter] = useState<EnabledFilterValue>('all')
  const [usageFilter, setUsageFilter] = useState<UsageFilterValue>('all')
  const [reasoningFilter, setReasoningFilter] = useState(false)
  const [autoRouterFilter, setAutoRouterFilter] = useState(false)
  const [oauthFilter, setOauthFilter] = useState(false)
  const [selectedProviders, setSelectedProviders] = useState<string[]>([])

  const allProviders = useMemo(() => {
    const providers = new Set(llmModels.map((model) => model.provider))
    return Array.from(providers).sort()
  }, [llmModels])

  const toggleProvider = (provider: string) => {
    setSelectedProviders((prev) => (prev.includes(provider) ? prev.filter((p) => p !== provider) : [...prev, provider]))
  }

  const minOutputPrice = useMemo(
    () => Math.min(...llmModels.map((model) => parseFloat(model.output_price))),
    [llmModels],
  )
  const maxOutputPrice = useMemo(
    () => Math.max(...llmModels.map((model) => parseFloat(model.output_price))),
    [llmModels],
  )

  const filteredAndSortedModels = useMemo(() => {
    let result = [...llmModels]

    if (searchQuery.trim()) {
      result = result.filter((model) => fuzzyMatch(model.name, searchQuery.trim()))
    }

    if (enabledFilter === 'enabled') {
      result = result.filter((model) => model.toggledOn)
    } else if (enabledFilter === 'disabled') {
      result = result.filter((model) => !model.toggledOn)
    }

    if (usageFilter !== 'all') {
      result = result.filter(
        (model) => getLabelForValue(minOutputPrice, maxOutputPrice, parseFloat(model.output_price)) === usageFilter,
      )
    }

    if (reasoningFilter) {
      result = result.filter((model) => model.supports_reasoning)
    }

    if (autoRouterFilter) {
      result = result.filter((model) => model.auto_router)
    }

    if (oauthFilter) {
      result = result.filter((model) => model.supports_oauth)
    }

    if (selectedProviders.length > 0) {
      result = result.filter((model) => selectedProviders.includes(model.provider))
    }

    result.sort((a, b) => {
      const multiplier = sort.direction === 'asc' ? 1 : -1

      let primaryCompare = 0
      switch (sort.column) {
        case 'enabled':
          primaryCompare = multiplier * (Number(a.toggledOn) - Number(b.toggledOn))
          break
        case 'name':
          return multiplier * a.name.localeCompare(b.name)
        case 'usage':
          primaryCompare = multiplier * (parseFloat(a.output_price) - parseFloat(b.output_price))
          break
        case 'context':
          primaryCompare = multiplier * Number(BigInt(a.context_length) - BigInt(b.context_length))
          break
        case 'reasoning':
          primaryCompare = multiplier * (Number(a.supports_reasoning) - Number(b.supports_reasoning))
          break
        case 'auto-router':
          primaryCompare = multiplier * (Number(a.auto_router) - Number(b.auto_router))
          break
        case 'oauth':
          primaryCompare = multiplier * (Number(a.supports_oauth) - Number(b.supports_oauth))
          break
      }

      if (primaryCompare === 0) {
        return a.name.localeCompare(b.name)
      }

      return primaryCompare
    })

    return result
  }, [
    llmModels,
    searchQuery,
    enabledFilter,
    usageFilter,
    reasoningFilter,
    autoRouterFilter,
    oauthFilter,
    selectedProviders,
    sort,
    minOutputPrice,
    maxOutputPrice,
  ])

  const handleSort = (column: SortColumn) => {
    setSort((prev) => ({
      column,
      direction: prev.column === column && prev.direction === 'desc' ? 'asc' : 'desc',
    }))
  }

  const hasActiveFilters =
    searchQuery || enabledFilter !== 'all' || usageFilter !== 'all' || reasoningFilter || selectedProviders.length > 0

  const clearFilters = () => {
    setSearchQuery('')
    setEnabledFilter('all')
    setUsageFilter('all')
    setReasoningFilter(false)
    setSelectedProviders([])
  }

  return (
    <Section>
      <SectionField
        title={
          <div className="flex flex-col gap-2">
            <div>Enabled Models</div>
            <div className="text-muted-foreground text-sm font-light">Select which models blprnt can use.</div>
            <div className="text-muted-foreground text-sm font-light italic">
              If none are selected, all models will be available.
            </div>
          </div>
        }
      >
        <div className="w-full space-y-3">
          <SearchInput value={searchQuery} onChange={setSearchQuery} />

          <div className="flex flex-wrap items-end gap-4">
            <StatusFilter value={enabledFilter} onChange={setEnabledFilter} />
            <UsageFilterSelect value={usageFilter} onChange={setUsageFilter} />

            <div className="flex flex-col gap-1.5">
              <span className="text-xs text-muted-foreground">Provider</span>
              <ProviderMultiSelect providers={allProviders} selected={selectedProviders} onToggle={toggleProvider} />
            </div>

            <ReasoningFilter checked={reasoningFilter} onChange={setReasoningFilter} />

            <AutoRouterFilter checked={autoRouterFilter} onChange={setAutoRouterFilter} />

            <OauthFilter checked={oauthFilter} onChange={setOauthFilter} />

            {hasActiveFilters && <ClearFiltersButton onClick={clearFilters} />}
          </div>

          <ResultsCount filtered={filteredAndSortedModels.length} total={llmModels.length} />

          <Table className="w-full" data-tour="user-account-models-table">
            <ModelsTableHeader sort={sort} onSort={handleSort} />
            <TableBody>
              {filteredAndSortedModels.length === 0 ? (
                <TableRow>
                  <TableCell className="text-center text-muted-foreground py-8" colSpan={6}>
                    No models match your filters
                  </TableCell>
                </TableRow>
              ) : (
                filteredAndSortedModels.map((model) => (
                  <ModelRow
                    key={model.slug}
                    maxOutputPrice={maxOutputPrice}
                    minOutputPrice={minOutputPrice}
                    model={model}
                    sortBy={sort.column}
                    sortDirection={sort.direction}
                    toggleSlug={toggleSlug}
                  />
                ))
              )}
            </TableBody>
          </Table>
        </div>
      </SectionField>
    </Section>
  )
}
