import { describe, expect, it } from 'vitest'
import { parseMarkdownDocument } from '@/lib/markdown'

const getHeadingTexts = (markdown: string) => {
  const document = parseMarkdownDocument(markdown)

  return (document.content ?? [])
    .filter((node) => node.type === 'heading')
    .map((node) => node.content?.map((child) => child.text ?? '').join('') ?? '')
}

describe('markdown preview parsing', () => {
  it('keeps heading text after ordered lists', () => {
    expect(getHeadingTexts('### A\n1. child\n\n### B')).toEqual(['A', 'B'])
  })

  it('keeps later section headings in implementation update notes', () => {
    expect(
      getHeadingTexts(`## CTO update

I converted this feature request into executable implementation tracks.

### What I changed
- Moved the parent issue to \`in_progress\` so it now reflects active architecture/coordination work.
- Created two child issues:
  1. **Implement issue comment mentions backend and run triggering** — unassigned for now because this work is primarily Rust backend/runtime and we do not currently have a dedicated backend/staff engineer assigned in the active roster.
  2. **Implement mention-aware issue comment composer and rendering** — assigned to **Frontend Engineer** so UI work can start against the documented contract.

### Current status
- Architecture/spec remains in \`docs/issue-comment-mentions.md\`.
- Frontend implementation can proceed now.
- Backend/runtime implementation is the critical remaining staffing gap for this feature.

### Recommended next step
- Have an engineering implementer pick up the backend/runtime child issue first or in parallel with frontend so the API contract lands early.
- Once backend is staffed, frontend can integrate against the shipped comment mention payload + \`issue_mention\` trigger behavior.

### Risk / blocker
- This feature crosses backend, runtime, frontend, docs, and tests. Without backend ownership, the parent issue cannot close even if the frontend child lands.`),
    ).toEqual(['CTO update', 'What I changed', 'Current status', 'Recommended next step', 'Risk / blocker'])
  })
})
