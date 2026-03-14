// blprntLoader.tsx
import { useId } from 'react'

type Props = {
  src?: string // PNG of the logo (transparent bg)
  size?: number // px
  color?: string // brand color
  bg?: string // blprnt fill
}

export const Loader = ({ src = '/images/icon.png', size = 225, color = '#0094f6', bg = '#071b2d' }: Props) => {
  const id = useId()
  const maskId = `${id}-mask`
  const outlineId = `${id}-outline`
  const shineId = `${id}-shine`
  const revealId = `${id}-reveal`
  const gridClipId = `${id}-grid`
  const speed = 3
  const alphaMaskId = `${id}-alpha-mask`
  const alphaThresholdId = `${id}-alpha-threshold`

  return (
    <svg aria-label="Loading" className="block" height={size} role="img" viewBox="0 0 500 500" width={size}>
      <defs>
        <mask id={maskId} maskUnits="userSpaceOnUse">
          <image height="500" href={src} width="500" x="0" y="0" />
        </mask>

        <filter height="140%" id={alphaThresholdId} width="140%" x="-20%" y="-20%">
          <feComponentTransfer>
            <feFuncA tableValues="0 0 0 1 1" type="table" />
          </feComponentTransfer>
        </filter>

        {/* Use the PNG alpha as a mask so the SVG animation keeps the exact silhouette */}
        <mask
          id={alphaMaskId}
          maskUnits="userSpaceOnUse"
          // Force alpha-based masking in all engines
          style={{ maskType: 'alpha' }}
        >
          <g filter={`url(#${alphaThresholdId})`}>
            <image height="500" href={src} width="500" x="0" y="0" />
          </g>
        </mask>

        {/* Pixel-perfect outline generated from the silhouette */}
        <filter height="140%" id={outlineId} width="140%" x="-20%" y="-20%">
          <feMorphology in="SourceAlpha" operator="dilate" radius="10" result="spread" />
          <feComposite in="spread" in2="SourceAlpha" operator="xor" result="ring" />
          <feFlood floodColor={color} result="c" />
          <feComposite in="c" in2="ring" operator="in" result="stroke" />
          <feGaussianBlur in="stroke" result="glow" stdDeviation="0.6" />
          <feMerge>
            <feMergeNode in="stroke" />
            <feMergeNode in="glow" />
          </feMerge>
        </filter>

        {/* Shine sweep for inside the shape */}
        <linearGradient id={shineId} x1="0" x2="1" y1="0" y2="0">
          <stop offset="0" stopColor="#fff" stopOpacity="0" />
          <stop offset="0.5" stopColor="#fff" stopOpacity="1" />
          <stop offset="1" stopColor="#fff" stopOpacity="0" />
        </linearGradient>

        {/* Animate initial unfurl from the left */}
        <clipPath id={revealId}>
          <rect height="500" width="0" x="0" y="0">
            <animate
              attributeName="width"
              dur={`${speed}s`}
              keyTimes="0;0.18;1"
              repeatCount="indefinite"
              values="0;500;500"
            />
          </rect>
        </clipPath>

        {/* Keep grid off the left band so it feels like the original icon */}
        <clipPath id={gridClipId}>
          <rect height="310" rx="8" width="300" x="175" y="95" />
        </clipPath>
      </defs>

      <g mask={`url(#${alphaMaskId})`}>
        <rect fill={bg} height="500" width="500" />

        <rect fill={`url(#${shineId})`} height="500" opacity="0.8" width="120" x="-180" y="0">
          <animate attributeName="x" dur={`${speed}s`} repeatCount="indefinite" values="-180;520" />
        </rect>
      </g>

      {/* Outline pulse */}
      <g filter={`url(#${outlineId})`} opacity="0.85">
        <image height="500" href={src} opacity="0" width="500" x="0" y="0">
          <animate attributeName="opacity" dur={`${speed}s`} repeatCount="indefinite" values="0;0.9;0.3;0.9;0" />
        </image>
      </g>

      {/* Inside: blprnt fill, grid draw, shine sweep */}

      <g clipPath={`url(#${revealId})`} mask={`url(#${maskId})`}>
        <rect fill={bg} height="500" width="500" />

        <rect fill={`url(#${shineId})`} height="500" opacity="0.8" width="120" x="-180" y="0">
          <animate attributeName="x" dur={`${speed}s`} repeatCount="indefinite" values="-180;520" />
        </rect>
      </g>

      {/* Subtle whole-logo breathe */}
      <g transform="translate(250 250)">
        <animateTransform
          additive="sum"
          attributeName="transform"
          dur={`${speed}s`}
          keyTimes="0;0.5;1"
          repeatCount="indefinite"
          type="scale"
          values="1,1;1.03,1.03;1,1"
        />
        <animateTransform
          additive="sum"
          attributeName="transform"
          dur={`${speed}s`}
          repeatCount="indefinite"
          type="translate"
          values="0 0;0 0"
        />
      </g>
    </svg>
  )
}
