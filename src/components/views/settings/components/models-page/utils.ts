export const fuzzyMatch = (text: string, query: string): boolean => {
  const textLower = text.toLowerCase()
  const queryLower = query.toLowerCase()

  let textIndex = 0
  for (const char of queryLower) {
    const foundIndex = textLower.indexOf(char, textIndex)
    if (foundIndex === -1) return false
    textIndex = foundIndex + 1
  }
  return true
}

const toHex = (n: number) => n.toString(16).padStart(2, '0')

const parseHex = (hex: string) => {
  const clean = hex.replace('#', '')
  return {
    b: parseInt(clean.substring(4, 6), 16),
    g: parseInt(clean.substring(2, 4), 16),
    r: parseInt(clean.substring(0, 2), 16),
  }
}

const startColor = parseHex('#00bf75')
const endColor = parseHex('#ff5464')

const getRatio = (min: number, max: number, value: number) => {
  const linearRatio = max === min ? 0 : Math.max(0, Math.min(1, (value - min) / (max - min)))
  return linearRatio ** 0.45
}

export const getColorForValue = (min: number, max: number, value: number) => {
  const ratio = getRatio(min, max, value)

  const r = Math.round(startColor.r + (endColor.r - startColor.r) * ratio)
  const g = Math.round(startColor.g + (endColor.g - startColor.g) * ratio)
  const b = Math.round(startColor.b + (endColor.b - startColor.b) * ratio)

  const hex = `#${toHex(r)}${toHex(g)}${toHex(b)}`

  return hex
}

export const USAGE_LABELS = ['Free', 'Minimal', 'Low', 'Medium', 'High', 'X-High'] as const
export type UsageLabel = (typeof USAGE_LABELS)[number]

const paidLabels = ['Minimal', 'Low', 'Medium', 'High', 'X-High']

export const getLabelForValue = (min: number, max: number, value: number): UsageLabel => {
  if (value === min || value === 0) return 'Free'

  const ratio = getRatio(min, max, value)

  const index = Math.min(Math.floor(ratio * paidLabels.length), paidLabels.length - 1)

  return paidLabels[index] as UsageLabel
}

export const getContrastingTextColor = (
  hexColor: string,
  lightColor = 'var(--foreground)',
  darkColor = 'var(--background)',
) => {
  const hex = hexColor.replace('#', '')
  const r = parseInt(hex.substring(0, 2), 16)
  const g = parseInt(hex.substring(2, 4), 16)
  const b = parseInt(hex.substring(4, 6), 16)

  const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255

  return luminance >= 0.49 ? darkColor : lightColor
}

export const parseContextLength = (contextLength: bigint) => {
  const contextLengthNumber = BigInt(contextLength)
  if (contextLengthNumber < 1000) return contextLengthNumber
  if (contextLengthNumber < 1000000) return `${(contextLengthNumber / BigInt(1000)).toLocaleString('en-US')}k`

  return `${(contextLengthNumber / BigInt(1000000)).toLocaleString('en-US')}m`
}
