import type { Employee } from '@/bindings/Employee'
import type { MentionPayload } from '@/bindings/MentionPayload'

export interface MentionSelection {
  employeeId: string
  label: string
  start: number
  end: number
}

export interface MentionQuery {
  query: string
  start: number
  end: number
}

export interface CommentSegment {
  kind: 'text' | 'mention'
  value: string
  employeeId?: string
}

interface MentionLinkTarget {
  employeeId: string
  label: string
}

interface IssueLinkTarget {
  issueId: string
  identifier: string
}

const mentionBoundaryPattern = /[\s([{-]/

export const mentionLabel = (employee: Employee) => employee.name

export const mentionToken = (label: string) => `@${label}`

export const getMentionQuery = (text: string, caret: number): MentionQuery | null => {
  if (caret < 0 || caret > text.length) return null

  const beforeCaret = text.slice(0, caret)
  const mentionStart = beforeCaret.lastIndexOf('@')
  if (mentionStart === -1) return null

  const boundary = mentionStart === 0 ? '' : text[mentionStart - 1]
  if (boundary && !mentionBoundaryPattern.test(boundary)) return null

  const query = text.slice(mentionStart + 1, caret)
  if (query.length === 0) return { end: caret, query: '', start: mentionStart }
  if (/\s$/.test(query) || query.includes('\n')) return null

  return { end: caret, query, start: mentionStart }
}

export const insertMentionSelection = (text: string, activeQuery: MentionQuery, employee: Employee) => {
  const label = mentionLabel(employee)
  const token = mentionToken(label)
  const nextText = `${text.slice(0, activeQuery.start)}${token} ${text.slice(activeQuery.end)}`
  const start = activeQuery.start
  const end = start + token.length

  return {
    nextCaret: end + 1,
    nextText,
    selection: { employeeId: employee.id, end, label, start } satisfies MentionSelection,
  }
}

export const filterMentionSuggestions = (employees: Employee[], query: string) => {
  const normalizedQuery = query.trim().toLowerCase()
  if (!normalizedQuery) return employees

  return [...employees]
    .sort((left, right) => {
      const leftRank = rankEmployee(left, normalizedQuery)
      const rightRank = rankEmployee(right, normalizedQuery)
      if (leftRank !== rightRank) return leftRank - rightRank
      return left.name.localeCompare(right.name)
    })
    .filter((employee) => rankEmployee(employee, normalizedQuery) < Number.POSITIVE_INFINITY)
}

export const getNextMentionSuggestionIndex = (currentIndex: number, suggestionCount: number, direction: 1 | -1) => {
  if (suggestionCount <= 0) return 0

  return (currentIndex + direction + suggestionCount) % suggestionCount
}

export const reconcileMentionSelections = (text: string, selections: MentionSelection[]) => {
  const usedRanges = new Set<string>()

  return [...selections]
    .sort((left, right) => left.start - right.start)
    .flatMap((selection) => {
      const token = mentionToken(selection.label)
      const matches = findTokenMatches(text, token)
      const nextMatch = matches
        .filter((match) => !usedRanges.has(`${match.start}:${match.end}`))
        .sort((left, right) => Math.abs(left.start - selection.start) - Math.abs(right.start - selection.start))[0]

      if (!nextMatch) return []

      usedRanges.add(`${nextMatch.start}:${nextMatch.end}`)

      return [
        {
          employeeId: selection.employeeId,
          end: nextMatch.end,
          label: selection.label,
          start: nextMatch.start,
        } satisfies MentionSelection,
      ]
    })
}

export const inferMentionSelections = (text: string, employees: Employee[], selections: MentionSelection[] = []) => {
  const reconciledSelections = reconcileMentionSelections(text, selections)
  const usedRanges = new Set(reconciledSelections.map((selection) => `${selection.start}:${selection.end}`))

  const inferredSelections = employees.flatMap((employee) => {
    const label = mentionLabel(employee)
    const token = mentionToken(label)

    return findTokenMatches(text, token)
      .filter((match) => isBoundariedMention(text, match.start, match.end))
      .filter((match) => !usedRanges.has(`${match.start}:${match.end}`))
      .map(
        (match) =>
          ({
            employeeId: employee.id,
            end: match.end,
            label,
            start: match.start,
          }) satisfies MentionSelection,
      )
  })

  return [...reconciledSelections, ...inferredSelections].sort((left, right) => left.start - right.start)
}

export const mentionPayloadsFromSelections = (selections: MentionSelection[]): MentionPayload[] => {
  const seen = new Set<string>()

  return selections.flatMap((selection) => {
    if (seen.has(selection.employeeId)) return []
    seen.add(selection.employeeId)
    return [{ employee_id: selection.employeeId, label: selection.label }]
  })
}

export const linkifyMentionsInMarkdown = (text: string, mentions: MentionLinkTarget[]) => {
  if (!text.trim() || mentions.length === 0) return text

  return mentions
    .sort((left, right) => right.label.length - left.label.length)
    .reduce((markdown, mention) => replaceMentionToken(markdown, mention), text)
}

export const linkifyEmployeeMentionsInMarkdown = (text: string, employees: Employee[]) => {
  return linkifyMentionsInMarkdown(
    text,
    employees.map((employee) => ({ employeeId: employee.id, label: mentionLabel(employee) })),
  )
}

export const linkifyIssueIdentifiersInMarkdown = (text: string, issues: IssueLinkTarget[]) => {
  if (!text.trim() || issues.length === 0) return text

  return issues
    .sort((left, right) => right.identifier.length - left.identifier.length)
    .reduce((markdown, issue) => replaceIssueIdentifier(replaceInlineCodeIssueIdentifier(markdown, issue), issue), text)
}

export const segmentCommentWithMentions = (text: string, mentions: MentionPayload[]): CommentSegment[] => {
  if (mentions.length === 0) return [{ kind: 'text', value: text }]

  const normalizedMentions = mentions
    .flatMap((mention) => {
      const token = mentionToken(mention.label)
      return findTokenMatches(text, token).map((match) => ({ ...match, employeeId: mention.employee_id, value: token }))
    })
    .sort((left, right) => left.start - right.start)
    .filter((mention, index, list) => index === 0 || mention.start >= list[index - 1].end)

  if (normalizedMentions.length === 0) return [{ kind: 'text', value: text }]

  const segments: CommentSegment[] = []
  let cursor = 0

  for (const mention of normalizedMentions) {
    if (mention.start > cursor) {
      segments.push({ kind: 'text', value: text.slice(cursor, mention.start) })
    }

    segments.push({ employeeId: mention.employeeId, kind: 'mention', value: mention.value })
    cursor = mention.end
  }

  if (cursor < text.length) {
    segments.push({ kind: 'text', value: text.slice(cursor) })
  }

  return segments
}

const rankEmployee = (employee: Employee, normalizedQuery: string) => {
  const name = employee.name.toLowerCase()
  if (name.startsWith(normalizedQuery)) return 0
  if (name.split(/\s+/).some((part) => part.startsWith(normalizedQuery))) return 1
  if (name.includes(normalizedQuery)) return 2
  return Number.POSITIVE_INFINITY
}

const findTokenMatches = (text: string, token: string) => {
  const matches: Array<{ start: number; end: number }> = []
  let cursor = 0

  while (cursor < text.length) {
    const start = text.indexOf(token, cursor)
    if (start === -1) break
    const end = start + token.length
    matches.push({ end, start })
    cursor = end
  }

  return matches
}

const isBoundariedMention = (text: string, start: number, end: number) => {
  const before = start === 0 ? '' : text[start - 1]
  const after = end >= text.length ? '' : text[end]

  const hasValidBoundaryBefore = before === '' || mentionBoundaryPattern.test(before)
  const hasValidBoundaryAfter = after === '' || /[\s)\]}.!?,:;-]/.test(after)

  return hasValidBoundaryBefore && hasValidBoundaryAfter
}

const replaceMentionToken = (markdown: string, mention: MentionLinkTarget) => {
  const token = mentionToken(mention.label)
  let result = ''
  let cursor = 0

  while (cursor < markdown.length) {
    const start = markdown.indexOf(token, cursor)
    if (start === -1) {
      result += markdown.slice(cursor)
      break
    }

    const end = start + token.length
    const before = start === 0 ? '' : markdown[start - 1]
    const after = end >= markdown.length ? '' : markdown[end]

    const insideMarkdownLink = start > 0 && markdown[start - 1] === '['
    const hasValidBoundaryBefore = before === '' || mentionBoundaryPattern.test(before)
    const hasValidBoundaryAfter = after === '' || /[\s)\]}.!?,:;-]/.test(after)

    result += markdown.slice(cursor, start)

    if (insideMarkdownLink || !hasValidBoundaryBefore || !hasValidBoundaryAfter) {
      result += token
    } else {
      result += `[${token}](/employees/${mention.employeeId})`
    }

    cursor = end
  }

  return result
}

const replaceIssueIdentifier = (markdown: string, issue: IssueLinkTarget) => {
  const token = issue.identifier
  let result = ''
  let cursor = 0

  while (cursor < markdown.length) {
    const start = markdown.indexOf(token, cursor)
    if (start === -1) {
      result += markdown.slice(cursor)
      break
    }

    const end = start + token.length
    const before = start === 0 ? '' : markdown[start - 1]
    const after = end >= markdown.length ? '' : markdown[end]

    const insideMarkdownLink = start > 0 && markdown[start - 1] === '['
    const hasValidBoundaryBefore = before === '' || /[\s([{-]/.test(before)
    const hasValidBoundaryAfter = after === '' || /[\s)\]}.!?,:;-]/.test(after)

    result += markdown.slice(cursor, start)

    if (insideMarkdownLink || !hasValidBoundaryBefore || !hasValidBoundaryAfter) {
      result += token
    } else {
      result += `[${token}](/issues/${issue.issueId})`
    }

    cursor = end
  }

  return result
}

const replaceInlineCodeIssueIdentifier = (markdown: string, issue: IssueLinkTarget) => {
  const escapedIdentifier = escapeRegExp(issue.identifier)
  return markdown.replace(
    new RegExp(`(^|[^\\[])\`${escapedIdentifier}\`(?=$|[\\s)\\]}.!?,:;-])`, 'g'),
    (_match, boundary: string) => `${boundary}[\`${issue.identifier}\`](/issues/${issue.issueId})`,
  )
}

const escapeRegExp = (value: string) => value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
