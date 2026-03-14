import type { ErrorEvent, Result } from '@/bindings'

export const asyncWrapper = <T>(promise: Promise<Result<T, ErrorEvent>>): Promise<[ErrorEvent, null] | [null, T]> =>
  new Promise((resolve) => {
    promise
      .then((result) => {
        if (result.status === 'error') {
          resolve([result.error, null])
        } else {
          resolve([null, result.data])
        }
      })
      .catch((error) => {
        resolve([error, null])
      })
  })
