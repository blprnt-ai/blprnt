// @vitest-environment jsdom
import dayjs from 'dayjs'
import type { Dayjs } from 'dayjs'
import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import type { ButtonHTMLAttributes, ReactNode, TextareaHTMLAttributes } from 'react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { SessionInput } from './session-input'

let activeViewmodel: ReturnType<typeof createViewmodelStub>

vi.mock('@/components/panels/session/session-panel.viewmodel', () => ({
  useSessionPanelViewmodel: () => activeViewmodel,
}))

vi.mock('framer-motion', () => ({
  AnimatePresence: ({ children }: { children: ReactNode }) => <>{children}</>,
  motion: {
    div: ({ children, ...props }: { children: ReactNode }) => <div {...props}>{children}</div>,
  },
}))

vi.mock('@/components/atoms/tooltip-macro', () => ({
  TooltipMacro: ({ children }: { children: ReactNode }) => <>{children}</>,
}))

vi.mock('@/components/atoms/button', () => ({
  Button: ({ children, ...props }: ButtonHTMLAttributes<HTMLButtonElement> & { children?: ReactNode }) => (
    <button {...props}>{children}</button>
  ),
}))

vi.mock('@/components/atoms/textarea', () => ({
  Textarea: ({ ...props }: TextareaHTMLAttributes<HTMLTextAreaElement>) => <textarea {...props} />,
}))

vi.mock('@/components/atoms/toaster', () => ({
  stopSessionToast: {
    loading: vi.fn(),
    success: vi.fn(),
  },
}))

type QueuedPromptItem = {
  id: string
  content: string
  imageUrls: string[]
  isDeleting: boolean
  createdAt: Dayjs
}

const createViewmodelStub = (queuedPrompts: QueuedPromptItem[]) => ({
  addImageUrl: vi.fn(),
  deleteQueuedPrompt: vi.fn().mockResolvedValue(undefined),
  imageUrls: new Map<string, string>(),
  interrupt: vi.fn().mockResolvedValue(undefined),
  isRunning: false,
  prompt: '',
  queuedPrompts,
  removeImageUrl: vi.fn(),
  session: { id: 'session-1' },
  setPrompt: vi.fn(),
  submitPrompt: vi.fn().mockResolvedValue(undefined),
})

afterEach(() => {
  cleanup()
})

describe('SessionInput queued inline delete UI', () => {
  it('renders queued row with inline delete layout and invokes delete action', async () => {
    const viewmodel = createViewmodelStub([
      {
        content: 'Queued item one',
        createdAt: dayjs(),
        id: 'queue-item-1',
        imageUrls: [],
        isDeleting: false,
      },
    ])
    activeViewmodel = viewmodel

    const { container } = render(<SessionInput />)

    const contentNode = screen.getByText('Queued item one')
    const inlineRow = contentNode.parentElement
    expect(inlineRow).not.toBeNull()
    expect(inlineRow?.className).toContain('flex')
    expect(inlineRow?.className).toContain('justify-between')
    expect(inlineRow?.className).toContain('items-start')

    const deleteButton = inlineRow?.querySelector('button')
    expect(deleteButton).not.toBeNull()
    expect(deleteButton?.hasAttribute('disabled')).toBe(false)

    fireEvent.click(deleteButton!)

    expect(viewmodel.deleteQueuedPrompt).toHaveBeenCalledTimes(1)
    expect(viewmodel.deleteQueuedPrompt).toHaveBeenCalledWith('queue-item-1')

    expect(container.textContent).toContain('Queue')
  })

  it('disables inline delete and shows spinner state while deleting', () => {
    const viewmodel = createViewmodelStub([
      {
        content: 'Deleting queued item',
        createdAt: dayjs(),
        id: 'queue-item-2',
        imageUrls: [],
        isDeleting: true,
      },
    ])
    activeViewmodel = viewmodel

    const { container } = render(<SessionInput />)

    const contentNode = screen.getByText('Deleting queued item')
    const inlineRow = contentNode.parentElement
    const deleteButton = inlineRow?.querySelector('button')
    expect(deleteButton).not.toBeNull()
    expect(deleteButton?.hasAttribute('disabled')).toBe(true)

    const spinner = container.querySelector('svg.animate-spin')
    expect(spinner).not.toBeNull()
  })
})