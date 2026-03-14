import { openUrl } from '@tauri-apps/plugin-opener'
import { type } from '@tauri-apps/plugin-os'
import type React from 'react'
import { useEffect, useState } from 'react'
import { Kbd, KbdGroup } from '@/components/atoms/kbd'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/atoms/tooltip'
import { cn } from '@/lib/utils/cn'

const isMac = type() === 'macos'

export function ExternalLink({ href, children }: { href: string; children: React.ReactNode }) {
  return href.startsWith('#') ? (
    <a href={href}>{children}</a>
  ) : (
    <ExternalLinkComponent href={href}>{children}</ExternalLinkComponent>
  )
}

const ExternalLinkComponent = ({ href, children }: { href: string; children: React.ReactNode }) => {
  const [isModifierPressed, setIsModifierPressed] = useState(false)

  useEffect(() => {
    const body = document.querySelector('body')!

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key?.toLowerCase() === (isMac ? 'meta' : 'control')) {
        setIsModifierPressed(true)
      }
    }

    const handleKeyUp = (event: KeyboardEvent) => {
      if (event.key?.toLowerCase() === (isMac ? 'meta' : 'control')) {
        setIsModifierPressed(false)
      }
    }

    body.addEventListener('keydown', handleKeyDown)
    body.addEventListener('keyup', handleKeyUp)

    return () => {
      body.removeEventListener('keydown', handleKeyDown)
      body.removeEventListener('keyup', handleKeyUp)
    }
  }, [])

  const handleClick = async (event: React.MouseEvent<HTMLAnchorElement>) => {
    event.preventDefault()
    if (!isModifierPressed) return

    await openUrl(href)
  }

  const shortCut = isMac ? '⌘' : 'Ctrl'

  return (
    <Tooltip delayDuration={500}>
      <TooltipTrigger tabIndex={-1}>
        <a
          className={cn('text-primary', !isModifierPressed ? 'cursor-default' : 'underline-offset-4 hover:underline')}
          href={href}
          onClick={handleClick}
          onContextMenu={(event) => {
            event.preventDefault()
            event.stopPropagation()
          }}
        >
          {children}
        </a>
      </TooltipTrigger>
      <TooltipContent side="right">
        <KbdGroup>
          <Kbd>{shortCut}</Kbd>
          <span>+</span>
          <Kbd>Click</Kbd>
        </KbdGroup>
      </TooltipContent>
    </Tooltip>
  )
}
