// eslint-disable-next-line boundaries/element-types
import { BlprntEventEnum, once } from '@/lib/events/lib'

export const listenOAuthCallback = () => {
  return new Promise<string[]>((resolve) => once(BlprntEventEnum.OAuthCallback, resolve))
}
