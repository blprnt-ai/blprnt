import { XIcon } from 'lucide-react'
import { useEffect, useRef } from 'react'
import { Button } from '@/components/atoms/button'
import { useSessionPanelViewmodel } from '@/components/panels/session/session-panel.viewmodel'
import { TerminalMessageModel } from '@/lib/models/messages/terminal-message.model'
import { asyncWait } from '@/lib/utils/misc'

export const Terminals = ({ messages }: { messages: TerminalMessageModel[] }) => {
  if (
    messages.length === 0 ||
    messages.every((message) => message.lines.filter((line) => line.trim().length > 0).length === 0)
  )
    return null

  return (
    <div className="border-t bg-accent p-2 px-3 text-sm">
      <div className="flex flex-col gap-4">
        {messages.map((message) => (
          <TerminalCard key={message.id} message={message} />
        ))}
      </div>
    </div>
  )
}

export const TerminalCard = ({ message }: { message: TerminalMessageModel }) => {
  const errorCount = useRef(0)
  const timeoutRef = useRef<number>(null)
  const viewmodel = useSessionPanelViewmodel()

  useEffect(() => {
    if (!(message instanceof TerminalMessageModel)) return

    const refresh = async () => {
      if (errorCount.current > 0) await asyncWait(Math.min(1000 * 2 ** errorCount.current, 10000))

      try {
        const hasLines = await viewmodel.getTerminalSnapshot(message.id, message.terminalId)
        if (!hasLines) throw new Error('No lines')

        errorCount.current = 0
      } catch {
        errorCount.current++
      }

      if (errorCount.current < 5) timeoutRef.current = setTimeout(refresh, 5000)
    }

    timeoutRef.current = setTimeout(refresh, 5000)

    return () => {
      if (timeoutRef.current) clearTimeout(timeoutRef.current)
    }
  }, [message, viewmodel])

  if (!(message instanceof TerminalMessageModel)) return null

  const lines = message.lines.filter((line) => line.trim().length > 0)
  if (lines.length === 0) return null

  return (
    <div className="border border-border rounded-md p-2 px-3 text-sm bg-slate-950 text-green-600 relative">
      <div className="absolute top-0 right-0">
        <Button
          className="hover:border-none hover:text-destructive"
          size="icon"
          variant="outline-ghost"
          onClick={() => viewmodel.closeTerminal(message.id, message.terminalId)}
        >
          <XIcon className="size-4" />
        </Button>
      </div>
      <pre className="whitespace-pre-wrap break-all">
        {lines.map((line, index) => (
          <div key={index}>{line}</div>
        ))}
      </pre>
    </div>
  )
}
