// @vitest-environment jsdom
import { render, screen } from '@testing-library/react'
import type { ReactNode } from 'react'
import { describe, expect, it, vi } from 'vitest'
import { WebSearchToggle } from '@/components/forms/session/sections/web-search-toggle'

vi.mock('@/components/atoms/tooltip-macro', () => ({
  TooltipMacro: ({ children }: { children: ReactNode }) => <>{children}</>,
}))

describe('OpenrouterWebSearch', () => {
  it('renders exact label text "Enable web search"', () => {
    render(<WebSearchToggle webSearchEnabled={false} onSetWebSearchEnabled={vi.fn()} />)

    expect(screen.getByText('Enable web search')).toBeTruthy()
  })
})
