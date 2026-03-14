import { init } from '@sentry/react'

export const initSentry = () =>
  init({
    dsn: 'https://679efe7892299d01356f151df8728cfb@o4510423017717760.ingest.us.sentry.io/4510423019356160',
    sendDefaultPii: true,
  })
