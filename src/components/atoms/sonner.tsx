import { Toaster as Sonner, type ToasterProps } from 'sonner'

export const Toaster = ({ ...props }: ToasterProps) => {
  return (
    <Sonner
      expand
      className="toaster group z-50"
      closeButton={false}
      theme="dark"
      visibleToasts={10}
      offset={{
        right: '1rem',
        top: '1rem',
      }}
      style={
        {
          '--border-radius': 'var(--radius)',
          '--normal-bg': 'var(--popover)',
          '--normal-border': 'var(--border)',
          '--normal-text': 'var(--popover-foreground)',
          // '--width': '500px',
        } as React.CSSProperties
      }
      toastOptions={{
        style: {
          width: '100%',
        },
      }}
      {...props}
    />
  )
}

export const LicenseToaster = ({ ...props }: ToasterProps) => {
  return (
    <Sonner
      expand
      className="toaster group z-50"
      closeButton={false}
      theme="dark"
      visibleToasts={10}
      style={
        {
          '--border-radius': 'var(--radius)',
          '--normal-bg': 'var(--popover)',
          '--normal-border': 'var(--border)',
          '--normal-text': 'var(--popover-foreground)',
          // '--width': '500px',
        } as React.CSSProperties
      }
      toastOptions={{
        style: {
          width: '100%',
        },
      }}
      {...props}
    />
  )
}
