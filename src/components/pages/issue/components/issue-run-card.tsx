import { RunSummaryCard } from '@/components/organisms/run-summary-card'
import type { RunSummaryModel } from '@/models/run-summary.model'

interface IssueRunCardProps {
  run: RunSummaryModel
  latestActivity?: string | null
}

export const IssueRunCard = ({ run, latestActivity }: IssueRunCardProps) => {
  return <RunSummaryCard latestActivity={latestActivity} run={run} />
}