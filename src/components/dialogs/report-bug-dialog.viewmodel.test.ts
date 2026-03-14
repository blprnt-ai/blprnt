// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { ReportBugScreenshotContract, ReportBugSubmitResponse } from '@/bindings'
import type { ReportBugSubmitRequestPayload } from '@/lib/models/report-bug.types'
import { ReportBugDialogViewModel } from './report-bug-dialog.viewmodel'

class MockFileReader {
  public result: string | ArrayBuffer | null = null
  public error: DOMException | null = null
  public onload: ((this: FileReader, event: ProgressEvent<FileReader>) => void) | null = null
  public onerror: ((this: FileReader, event: ProgressEvent<FileReader>) => void) | null = null

  readAsDataURL(file: Blob) {
    const mimeType = (file as File).type || 'application/octet-stream'
    this.result = `data:${mimeType};base64,WA==`
    this.onload?.call(this as unknown as FileReader, new ProgressEvent('load') as ProgressEvent<FileReader>)
  }
}

describe('ReportBugDialogViewModel submit payload', () => {
  const screenshotContract: ReportBugScreenshotContract = {
    max_bytes: 0n,
    allowed_mime_types: [],
    allowed_reference_schemes: [],
    supported_kinds: ['inline_base64'],
  }

  const originalFileReader = globalThis.FileReader

  beforeEach(() => {
    vi.stubGlobal('FileReader', MockFileReader)
  })

  afterEach(() => {
    globalThis.FileReader = originalFileReader
    vi.restoreAllMocks()
  })

  it('submits screenshot and pasted attachments together with unchanged API payload shape', async () => {
    const submit = vi.fn<
      (request: ReportBugSubmitRequestPayload) => Promise<ReportBugSubmitResponse>
    >(async () => ({
      state: 'submitted',
      normalized_submission: null,
      error: null,
      screenshot_contract: screenshotContract,
    }))

    const viewmodel = new ReportBugDialogViewModel({
      submit,
    })

    const screenshot = new File(['screenshot-bytes'], 'screenshot.png', { type: 'image/png' })
    const attachment = new File(['attachment-bytes'], 'attachment.txt', { type: 'text/plain' })

    viewmodel.setTitle('Bug title')
    viewmodel.setDescription('Bug description')
    viewmodel.setSeverity('HIGH')
    viewmodel.screenshotFile = screenshot
    viewmodel.queuePastedFiles([attachment])

    await viewmodel.submit()

    expect(submit).toHaveBeenCalledTimes(1)
    const request = submit.mock.calls[0][0]

    expect(typeof request.screenshot?.byte_len).toBe('number')
    expect(request.screenshot?.byte_len).toBe(screenshot.size)
    expect(typeof request.pasted_attachments?.[0]?.byte_len).toBe('number')
    expect(request.pasted_attachments?.[0]?.byte_len).toBe(attachment.size)
    expect(request.screenshot).toEqual({
      kind: 'inline_base64',
      file_name: 'screenshot.png',
      mime_type: 'image/png',
      byte_len: screenshot.size,
      data_base64: 'WA==',
      reference_url: null,
    })
    expect(request.pasted_attachments).toEqual([
      {
        client_id: 'pasted-attachment-1',
        kind: 'file',
        file_name: 'attachment.txt',
        mime_type: 'text/plain',
        byte_len: attachment.size,
        payload_kind: 'inline_base64',
        data_base64: 'WA==',
        file_path: null,
        reference_url: null,
      },
    ])
    expect(() => JSON.stringify(request)).not.toThrow()
    expect(viewmodel.submitResult).toBe('success')
  })

  it('keeps retry behavior unchanged for retryable API rejection', async () => {
    const submit = vi
      .fn<(request: ReportBugSubmitRequestPayload) => Promise<ReportBugSubmitResponse>>()
      .mockResolvedValueOnce({
        state: 'rejected',
        normalized_submission: null,
        error: {
          code: 'RB_GITHUB_RATE_LIMITED',
          category: 'github',
          message: 'retry later',
          retryable: true,
          field_errors: [],
        },
        screenshot_contract: screenshotContract,
      })
      .mockResolvedValueOnce({
        state: 'submitted',
        normalized_submission: null,
        error: null,
        screenshot_contract: screenshotContract,
      })

    const viewmodel = new ReportBugDialogViewModel({
      submit,
    })

    viewmodel.setTitle('Bug title')
    viewmodel.setDescription('Bug description')
    viewmodel.setSeverity('HIGH')

    await viewmodel.submit()

    expect(viewmodel.submitResult).toBe('error')
    expect(viewmodel.canRetry).toBe(true)

    await viewmodel.retry()

    expect(submit).toHaveBeenCalledTimes(2)
    expect(viewmodel.submitResult).toBe('success')
    expect(viewmodel.canRetry).toBe(false)
  })

  it('keeps cancel behavior unchanged by clearing form and closing dialog', async () => {
    const submit = vi.fn<(request: ReportBugSubmitRequestPayload) => Promise<ReportBugSubmitResponse>>(async () => ({
      state: 'rejected',
      normalized_submission: null,
      error: {
        code: 'RB_GITHUB_RATE_LIMITED',
        category: 'github',
        message: 'retry later',
        retryable: true,
        field_errors: [],
      },
      screenshot_contract: screenshotContract,
    }))

    const viewmodel = new ReportBugDialogViewModel({
      submit,
    })

    viewmodel.open()
    viewmodel.setTitle('Bug title')
    viewmodel.setDescription('Bug description')
    viewmodel.setSeverity('HIGH')
    viewmodel.screenshotFile = new File(['screenshot-bytes'], 'screenshot.png', { type: 'image/png' })
    viewmodel.queuePastedFiles([new File(['attachment-bytes'], 'attachment.txt', { type: 'text/plain' })])

    await viewmodel.submit()

    expect(viewmodel.submitResult).toBe('error')
    expect(viewmodel.canRetry).toBe(true)

    viewmodel.cancel()

    expect(viewmodel.isOpen).toBe(false)
    expect(viewmodel.submitResult).toBe('idle')
    expect(viewmodel.errorMessage).toBeNull()
    expect(viewmodel.title).toBe('')
    expect(viewmodel.description).toBe('')
    expect(viewmodel.severity).toBe('')
    expect(viewmodel.screenshotFile).toBeNull()
    expect(viewmodel.queuedPastedAttachments).toHaveLength(0)
    expect(viewmodel.rejectedPastedAttachments).toHaveLength(0)
    expect(viewmodel.canRetry).toBe(false)
  })

  it('works without a status command', async () => {
    const submittedResponse: ReportBugSubmitResponse = {
      state: 'submitted',
      normalized_submission: null,
      error: null,
      screenshot_contract: screenshotContract,
    }

    const viewmodel = new ReportBugDialogViewModel({
      submit: vi.fn(async () => submittedResponse),
    })

    viewmodel.setTitle('Bug title')
    viewmodel.setDescription('Bug description')
    viewmodel.setSeverity('HIGH')

    await viewmodel.submit()

    expect(viewmodel.submitResult).toBe('success')
  })

  it('maps config submit errors to support-oriented messaging', async () => {
    const rejectedConfigInvalidResponse: ReportBugSubmitResponse = {
      state: 'rejected',
      normalized_submission: null,
      error: {
        code: 'RB_CONFIG_INVALID',
        category: 'config',
        message: 'legacy local config error',
        retryable: false,
        field_errors: [],
      },
      screenshot_contract: screenshotContract,
    }

    const viewmodel = new ReportBugDialogViewModel({
      submit: vi.fn(async () => rejectedConfigInvalidResponse),
    })

    viewmodel.setTitle('Bug title')
    viewmodel.setDescription('Bug description')
    viewmodel.setSeverity('HIGH')

    await viewmodel.submit()

    expect(viewmodel.submitResult).toBe('error')
    expect(viewmodel.errorMessage).toBe('Bug report setup is unavailable. Contact support.')
  })
})