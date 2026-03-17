import type { MermaidConfig } from 'mermaid'
import { useEffect, useState } from 'react'
import { cn } from '@/lib/utils/cn'
import { reportError } from '@/lib/utils/error-reporting'

const initializeMermaid = async (customConfig?: MermaidConfig) => {
  const defaultConfig: MermaidConfig = {
    fontFamily: 'monospace',
    securityLevel: 'strict',
    startOnLoad: false,
    suppressErrorRendering: true,
    theme: 'default',
  } as MermaidConfig

  const config = { ...defaultConfig, ...customConfig }

  const mermaidModule = await import('mermaid')
  const mermaid = mermaidModule.default

  mermaid.initialize(config)

  return mermaid
}

type MermaidProps = {
  chart: string
  className?: string
  config?: MermaidConfig
}

export const Mermaid = ({ chart, className, config }: MermaidProps) => {
  const [error, setError] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [svgContent, setSvgContent] = useState<string>('')
  const [lastValidSvg, setLastValidSvg] = useState<string>('')

  // biome-ignore lint/correctness/useExhaustiveDependencies: "Required for Mermaid"
  useEffect(() => {
    const renderChart = async () => {
      try {
        setError(null)
        setIsLoading(true)

        const mermaid = await initializeMermaid(config)

        const chartHash = chart.split('').reduce((acc, char) => {
          return ((acc << 5) - acc + char.charCodeAt(0)) | 0
        }, 0)
        const uniqueId = `mermaid-${Math.abs(chartHash)}-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`

        const { svg } = await mermaid.render(uniqueId, chart)

        setSvgContent(svg)
        setLastValidSvg(svg)
      } catch (err) {
        reportError(err, 'rendering mermaid chart')
        if (!(lastValidSvg || svgContent)) {
          const errorMessage = err instanceof Error ? err.message : 'Failed to render Mermaid chart'
          setError(errorMessage)
        }
      } finally {
        setIsLoading(false)
      }
    }

    renderChart()
  }, [chart, config])

  if (isLoading && !svgContent && !lastValidSvg) {
    return (
      <div className={cn('my-4 flex justify-center p-4', className)}>
        <div className="flex items-center space-x-2 text-muted-foreground">
          <div className="h-4 w-4 animate-spin rounded-full border-current border-b-2" />
          <span className="text-sm">Loading diagram...</span>
        </div>
      </div>
    )
  }

  if (error && !svgContent && !lastValidSvg) {
    return (
      <div className={cn('rounded-lg border border-red-200 bg-red-50 p-4', className)}>
        <p className="font-mono text-red-700 text-sm">Mermaid Error: {error}</p>
        <details className="mt-2">
          <summary className="cursor-pointer text-red-600 text-xs">Show Code</summary>
          <pre className="mt-2 overflow-x-auto rounded bg-red-100 p-2 text-red-800 text-xs">{chart}</pre>
        </details>
      </div>
    )
  }

  const displaySvg = svgContent || lastValidSvg

  return (
    <div
      aria-label="Mermaid chart"
      className={cn('my-4 flex justify-center', className)}
      // biome-ignore lint/security/noDangerouslySetInnerHtml: "Required for Mermaid"
      dangerouslySetInnerHTML={{ __html: displaySvg }}
      role="img"
    />
  )
}
