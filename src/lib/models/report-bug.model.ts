import type { ReportBugSubmitResponse } from '@/bindings'
import { AppApi } from '@/lib/api/app.api'
import type { ReportBugSubmitRequestPayload } from './report-bug.types'

export class ReportBugModel {
  private readonly api: Pick<AppApi, 'reportBugSubmit'>

  constructor(api: Pick<AppApi, 'reportBugSubmit'> = new AppApi()) {
    this.api = api
  }

  submit = async (request: ReportBugSubmitRequestPayload): Promise<ReportBugSubmitResponse> => {
    return this.api.reportBugSubmit(request)
  }
}