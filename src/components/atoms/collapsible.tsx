import * as CollapsiblePrimitive from '@radix-ui/react-collapsible'

export function Collapsible({ ...props }: React.ComponentProps<typeof CollapsiblePrimitive.Root>) {
  return <CollapsiblePrimitive.Root data-slot="collapsible" {...props} />
}

export function CollapsibleTrigger({ ...props }: React.ComponentProps<typeof CollapsiblePrimitive.Trigger>) {
  return <CollapsibleTrigger data-slot="collapsible-trigger" {...props} />
}

export function CollapsibleContent({ ...props }: React.ComponentProps<typeof CollapsiblePrimitive.Content>) {
  return <CollapsibleContent data-slot="collapsible-content" {...props} />
}
