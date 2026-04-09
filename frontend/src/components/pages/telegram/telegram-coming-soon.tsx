import { MessageCircleIcon } from 'lucide-react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'

export const TelegramComingSoon = () => {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <MessageCircleIcon className="size-4" />
          Telegram
        </CardTitle>
        <CardDescription>Coming soon.</CardDescription>
      </CardHeader>

      <CardContent className="text-sm text-muted-foreground">
        The Telegram setup flow is temporarily hidden from the product surface.
      </CardContent>
    </Card>
  )
}
