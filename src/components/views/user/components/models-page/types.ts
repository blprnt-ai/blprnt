export type SortColumn = 'enabled' | 'name' | 'usage' | 'context' | 'reasoning' | 'auto-router' | 'oauth'
export type SortDirection = 'asc' | 'desc'

export interface SortState {
  column: SortColumn
  direction: SortDirection
}
