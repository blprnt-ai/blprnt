import { type ComponentProps, memo } from 'react'
import type { BundledTheme } from 'shiki'
import synthwave84 from 'shiki/dist/themes/synthwave-84.mjs'
import { Streamdown } from 'streamdown'
import { cn } from '@/lib/utils/cn'

type ResponseProps = ComponentProps<typeof Streamdown>

export const Response = memo(
  ({ className, ...props }: ResponseProps) => (
    <Streamdown
      shikiTheme={[synthwave84 as unknown as BundledTheme, synthwave84 as unknown as BundledTheme]}
      className={cn(
        'size-full [&>*:first-child]:mt-0 [&>*:last-child]:mb-0',
        '[&_ol]:list-decimal! [&_ol]:pl-4',
        '[&_ul]:list-disc! [&_ul]:pl-4',
        className,
      )}
      controls={{
        code: false,
        table: false,
      }}
      {...props}
    />
  ),
  (prevProps, nextProps) => prevProps.children === nextProps.children,
)

Response.displayName = 'Response'
