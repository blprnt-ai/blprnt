import type {
  ReportBugPastedAttachmentPayload,
  ReportBugScreenshotPayload,
  ReportBugSubmitRequest,
} from '@/bindings'

type JsonSafeScreenshotPayload = Omit<ReportBugScreenshotPayload, 'byte_len'> & { byte_len: number }
type JsonSafePastedAttachmentPayload = Omit<ReportBugPastedAttachmentPayload, 'byte_len'> & {
  byte_len: number
  client_id?: string
}

export type ReportBugSubmitRequestPayload = Omit<ReportBugSubmitRequest, 'screenshot' | 'pasted_attachments'> & {
  screenshot: JsonSafeScreenshotPayload | null
  pasted_attachments?: JsonSafePastedAttachmentPayload[]
}