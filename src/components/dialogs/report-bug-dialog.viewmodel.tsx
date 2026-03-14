import { flow, makeAutoObservable } from 'mobx'
import type {
  ReportBugAttachmentPayloadKind,
  ReportBugErrorCode,
  ReportBugPastedAttachmentKind,
  ReportBugSeverity,
  ReportBugSubmitResponse,
} from '@/bindings'
import { EventType, globalEventBus } from '@/lib/events'
import { ReportBugModel } from '@/lib/models/report-bug.model'
import type { ReportBugSubmitRequestPayload } from '@/lib/models/report-bug.types'
import { basicToast } from '@/components/atoms/toaster'

const REPORT_BUG_SEVERITY_VALUES = ['LOW', 'MEDIUM', 'HIGH', 'CRITICAL'] as const
const MAX_PASTED_ATTACHMENT_BYTES = 25 * 1024 * 1024
const INLINE_BASE64_PAYLOAD_KIND: ReportBugAttachmentPayloadKind = 'inline_base64'

export type ReportBugQueuedAttachment = {
  clientId: string
  kind: ReportBugPastedAttachmentKind
  file: File
  fileName: string
  mimeType: string
  byteLen: number
}

export type ReportBugRejectedAttachment = {
  clientId: string
  fileName: string
  byteLen: number
  message: string
}

export type ReportBugSubmitResult = 'idle' | 'submitting' | 'success' | 'error'

type ValidationErrors = {
  description: string | null
  severity: string | null
  title: string | null
}

const DEFAULT_VALIDATION_ERRORS: ValidationErrors = {
  description: null,
  severity: null,
  title: null,
}

const normalizeText = (value: string) => value.trim()

const getTextError = (value: string, fieldLabel: string) => {
  if (normalizeText(value).length === 0) {
    return `${fieldLabel} is required`
  }

  return null
}

const isReportBugSeverity = (value: string): value is ReportBugSeverity =>
  REPORT_BUG_SEVERITY_VALUES.includes(value as ReportBugSeverity)

const REPORT_BUG_ERROR_MESSAGES: Partial<Record<ReportBugErrorCode, string>> = {
  RB_CONFIG_MISSING: 'Bug report setup is unavailable. Contact support.',
  RB_CONFIG_INVALID: 'Bug report setup is unavailable. Contact support.',
  RB_CONFIG_STORE_UNAVAILABLE: 'Bug report setup is temporarily unavailable. Retry shortly.',
  RB_VALIDATION_FAILED: 'Submitted data was rejected. Review fields and retry.',
  RB_SCREENSHOT_CONTRACT_VIOLATION: 'Screenshot is invalid. Remove or replace it and retry.',
  RB_ATTACHMENT_CONTRACT_VIOLATION: 'Attachment data is invalid. Remove and paste attachments again.',
  RB_ATTACHMENT_UPLOAD_CONFIG_INVALID: 'Attachment upload destination is invalid. Contact support.',
  RB_ATTACHMENT_UPLOAD_PERMISSION_DENIED: 'Attachment upload permission denied. Contact support.',
  RB_ATTACHMENT_UPLOAD_RATE_LIMITED: 'Attachment upload is rate limited. Retry shortly.',
  RB_ATTACHMENT_UPLOAD_FAILED: 'One or more attachments failed to upload. Bug report was not submitted. Retry to re-upload.',
  RB_GITHUB_AUTH_FAILED: 'Bug report authentication failed. Contact support.',
  RB_GITHUB_PERMISSION_DENIED: 'Bug report permission denied. Contact support.',
  RB_GITHUB_RATE_LIMITED: 'Bug report service is rate limited. Retry shortly.',
  RB_GITHUB_NOT_FOUND: 'Bug report destination was not found. Contact support.',
  RB_GITHUB_API_ERROR: 'Bug report service returned an error. Retry shortly.',
  RB_GITHUB_NETWORK_ERROR: 'Network error while submitting bug report. Retry shortly.',
  RB_GITHUB_RESPONSE_INVALID: 'Bug report service returned an invalid response. Retry shortly.',
  RB_SUBMIT_NOT_IMPLEMENTED: 'Bug report submission is unavailable.',
}

const toErrorMessage = (error: unknown) => {
  if (error instanceof Error && error.message.trim().length > 0) {
    return error.message
  }

  return 'Request failed.'
}

const fileToBase64 = (file: File): Promise<string> => {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()

    reader.onload = () => {
      if (typeof reader.result !== 'string') {
        reject(new Error('Failed to read screenshot payload.'))
        return
      }

      const separatorIndex = reader.result.indexOf(',')
      resolve(separatorIndex >= 0 ? reader.result.slice(separatorIndex + 1) : reader.result)
    }

    reader.onerror = () => {
      reject(reader.error ?? new Error('Failed to read screenshot payload.'))
    }

    reader.readAsDataURL(file)
  })
}

export class ReportBugDialogViewModel {
  public isOpen = false
  public title = ''
  public description = ''
  public severity: ReportBugSeverity | '' = ''
  public screenshotFile: File | null = null
  public screenshotPreviewUrl: string | null = null
  public queuedPastedAttachments: ReportBugQueuedAttachment[] = []
  public rejectedPastedAttachments: ReportBugRejectedAttachment[] = []
  public submitResult: ReportBugSubmitResult = 'idle'
  public errorMessage: string | null = null
  public errorCode: ReportBugErrorCode | null = null
  public isRetryableError = false
  public validationErrors: ValidationErrors = { ...DEFAULT_VALIDATION_ERRORS }
  public hasAttemptedSubmit = false

  private readonly model: Pick<ReportBugModel, 'submit'>
  private unsubscriber: (() => void) | null = null
  private submitAttempt = 0
  private attachmentCounter = 0

  constructor(model: Pick<ReportBugModel, 'submit'> = new ReportBugModel()) {
    this.model = model
    makeAutoObservable(this, {}, { autoBind: true })
  }

  init = flow(function* (this: ReportBugDialogViewModel) {
    if (this.unsubscriber) return

    this.unsubscriber = globalEventBus.subscribe(EventType.ReportBugMenuClicked, () => {
      void this.handleMenuOpenRequest()
    })
  })

  destroy = () => {
    this.unsubscriber?.()
    this.unsubscriber = null
    this.revokeScreenshotPreview()
  }

  handleMenuOpenRequest = flow(function* (this: ReportBugDialogViewModel) {
    this.open()
  })

  open = () => {
    this.isOpen = true
    this.submitResult = 'idle'
    this.errorMessage = null
    this.errorCode = null
    this.isRetryableError = false
    this.hasAttemptedSubmit = false
  }

  close = () => {
    this.cancel()
  }

  onOpenChange = (isOpen: boolean) => {
    if (!isOpen) {
      this.cancel()
      return
    }

    this.isOpen = true
  }

  setTitle = (value: string) => {
    this.title = value
    this.validationErrors.title = getTextError(value, 'Title')
  }

  setDescription = (value: string) => {
    this.description = value
    this.validationErrors.description = getTextError(value, 'Description')
  }

  setSeverity = (value: string) => {
    if (!isReportBugSeverity(value)) {
      this.severity = ''
      this.validationErrors.severity = 'Severity is required'
      return
    }

    this.severity = value
    this.validationErrors.severity = null
  }

  setScreenshotFile = (file: File | null) => {
    this.revokeScreenshotPreview()
    this.screenshotFile = file
    this.screenshotPreviewUrl = file ? URL.createObjectURL(file) : null
  }

  removeScreenshot = () => {
    this.setScreenshotFile(null)
  }

  queuePastedFiles = (files: File[]) => {
    for (const file of files) {
      const clientId = this.createAttachmentClientId()
      if (file.size > MAX_PASTED_ATTACHMENT_BYTES) {
        this.rejectedPastedAttachments.push({
          clientId,
          fileName: file.name,
          byteLen: file.size,
          message: 'File exceeds 25MB limit and was not added.',
        })
        continue
      }

      this.queuedPastedAttachments.push({
        clientId,
        kind: file.type.startsWith('image/') ? 'image' : 'file',
        file,
        fileName: file.name,
        mimeType: file.type,
        byteLen: file.size,
      })
    }
  }

  removeQueuedPastedAttachment = (clientId: string) => {
    this.queuedPastedAttachments = this.queuedPastedAttachments.filter((attachment) => attachment.clientId !== clientId)
  }

  dismissRejectedPastedAttachment = (clientId: string) => {
    this.rejectedPastedAttachments = this.rejectedPastedAttachments.filter((attachment) => attachment.clientId !== clientId)
  }

  submit = flow(function* (this: ReportBugDialogViewModel) {
    if (this.isSubmitting) {
      return
    }

    this.hasAttemptedSubmit = true
    this.touchValidation()
    if (!this.isFormValid) {
      this.submitResult = 'idle'
      return
    }

    this.submitResult = 'submitting'
    this.errorMessage = null
    this.errorCode = null
    this.isRetryableError = false
    const currentSubmitAttempt = ++this.submitAttempt

    try {
      const request: ReportBugSubmitRequestPayload = yield this.toSubmitRequest()
      const response: ReportBugSubmitResponse = yield this.model.submit(request)

      if (currentSubmitAttempt !== this.submitAttempt) {
        return
      }

      if (response.state === 'submitted') {
        this.submitResult = 'success'
        basicToast.success({
          description: 'Bug report submitted successfully.',
          title: 'Report sent',
        })
        this.cancel()
        return
      }

      this.submitResult = 'error'
      this.errorCode = response.error?.code ?? null
      this.isRetryableError = response.error?.retryable ?? false
      this.errorMessage = this.getSafeErrorMessage(response.error?.code, response.error?.message)
    } catch (error) {
      if (currentSubmitAttempt !== this.submitAttempt) {
        return
      }

      this.submitResult = 'error'
      this.isRetryableError = true
      this.errorMessage = toErrorMessage(error)
    }
  })

  retry = flow(function* (this: ReportBugDialogViewModel) {
    if (!this.canRetry) return
    yield this.submit()
  })

  cancel = () => {
    this.submitAttempt += 1
    this.isOpen = false
    this.submitResult = 'idle'
    this.errorMessage = null
    this.errorCode = null
    this.isRetryableError = false
    this.hasAttemptedSubmit = false
    this.validationErrors = { ...DEFAULT_VALIDATION_ERRORS }
    this.title = ''
    this.description = ''
    this.severity = ''
    this.removeScreenshot()
    this.queuedPastedAttachments = []
    this.rejectedPastedAttachments = []
  }

  resetAfterSuccess = () => {
    this.title = ''
    this.description = ''
    this.severity = ''
    this.removeScreenshot()
    this.queuedPastedAttachments = []
    this.rejectedPastedAttachments = []
    this.validationErrors = { ...DEFAULT_VALIDATION_ERRORS }
    this.submitResult = 'idle'
    this.errorMessage = null
    this.errorCode = null
    this.isRetryableError = false
    this.hasAttemptedSubmit = false
  }

  get isSubmitting() {
    return this.submitResult === 'submitting'
  }

  get isSuccess() {
    return this.submitResult === 'success'
  }

  get isError() {
    return this.submitResult === 'error'
  }

  get isInvalid() {
    return !this.isFormValid
  }

  get shouldShowInvalidBanner() {
    return this.hasAttemptedSubmit && this.isInvalid
  }

  get isFormValid() {
    return (
      getTextError(this.title, 'Title') === null &&
      getTextError(this.description, 'Description') === null &&
      isReportBugSeverity(this.severity)
    )
  }

  get canSubmit() {
    return this.isFormValid && !this.isSubmitting
  }

  get canRetry() {
    return this.isError && this.isRetryableError && !this.isSubmitting
  }

  private touchValidation = () => {
    this.validationErrors = {
      description: getTextError(this.description, 'Description'),
      severity: isReportBugSeverity(this.severity) ? null : 'Severity is required',
      title: getTextError(this.title, 'Title'),
    }
  }

  private revokeScreenshotPreview = () => {
    if (!this.screenshotPreviewUrl) return
    URL.revokeObjectURL(this.screenshotPreviewUrl)
  }

  private toSubmitRequest = async (): Promise<ReportBugSubmitRequestPayload> => {
    if (!isReportBugSeverity(this.severity)) {
      throw new Error('Severity is required')
    }

    const screenshot: ReportBugSubmitRequestPayload['screenshot'] = this.screenshotFile
      ? {
          kind: 'inline_base64',
          file_name: this.screenshotFile.name,
          mime_type: this.screenshotFile.type,
          byte_len: this.screenshotFile.size,
          data_base64: await fileToBase64(this.screenshotFile),
          reference_url: null,
        }
      : null

    const pastedAttachments: NonNullable<ReportBugSubmitRequestPayload['pasted_attachments']> = []
    for (const attachment of this.queuedPastedAttachments) {
      pastedAttachments.push({
        client_id: attachment.clientId,
        kind: attachment.kind,
        file_name: attachment.fileName,
        mime_type: attachment.mimeType,
        byte_len: attachment.byteLen,
        payload_kind: INLINE_BASE64_PAYLOAD_KIND,
        data_base64: await fileToBase64(attachment.file),
        file_path: null,
        reference_url: null,
      })
    }

    return {
      title: normalizeText(this.title),
      description: normalizeText(this.description),
      severity: this.severity,
      screenshot,
      pasted_attachments: pastedAttachments.length > 0 ? pastedAttachments : undefined,
    }
  }

  private createAttachmentClientId = () => {
    this.attachmentCounter += 1
    return `pasted-attachment-${this.attachmentCounter}`
  }

  private getSafeErrorMessage = (code: ReportBugErrorCode | null | undefined, fallbackMessage: string | null | undefined) => {
    if (code && REPORT_BUG_ERROR_MESSAGES[code]) {
      return REPORT_BUG_ERROR_MESSAGES[code]
    }

    if (fallbackMessage && fallbackMessage.trim().length > 0) {
      return fallbackMessage
    }

    return 'Bug report submission failed.'
  }
}