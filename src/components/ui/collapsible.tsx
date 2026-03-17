'use client'

import { Collapsible as CollapsiblePrimitive } from '@base-ui/react/collapsible'

export const Collapsible = ({ ...props }: CollapsiblePrimitive.Root.Props) => {
  return <CollapsiblePrimitive.Root data-slot="collapsible" {...props} />
}

export const CollapsibleTrigger = ({ ...props }: CollapsiblePrimitive.Trigger.Props) => {
  return <CollapsiblePrimitive.Trigger data-slot="collapsible-trigger" {...props} />
}

export const CollapsibleContent = ({ ...props }: CollapsiblePrimitive.Panel.Props) => {
  return <CollapsiblePrimitive.Panel data-slot="collapsible-content" {...props} />
}
