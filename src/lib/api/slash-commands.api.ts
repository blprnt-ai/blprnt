import type { SlashCommand } from '@/lib/models/slash-command.types'
// eslint-disable-next-line
import { asyncWait } from '@/lib/utils/misc'

const MOCK_DELAY_MS = 150
const DEV_COMMANDS: SlashCommand[] = [
  {
    description: 'Map the codebase structure and key modules.',
    keywords: ['structure', 'modules', 'index'],
    name: 'map-codebase',
  },
  {
    description: 'Update the project primer with new context.',
    keywords: ['context', 'summary', 'notes'],
    name: 'update-primer',
  },
  {
    description: 'Create a new agent personality template.',
    keywords: ['persona', 'profile', 'template'],
    name: 'create-personality',
  },
]

export const listSlashCommands = async (): Promise<SlashCommand[]> => {
  await asyncWait(MOCK_DELAY_MS)
  return DEV_COMMANDS
}
