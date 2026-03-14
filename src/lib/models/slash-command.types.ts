export type SlashCommandId = 'map-codebase' | 'update-primer' | 'create-personality'

export interface SlashCommand {
  name: SlashCommandId
  description: string
  keywords?: string[]
}