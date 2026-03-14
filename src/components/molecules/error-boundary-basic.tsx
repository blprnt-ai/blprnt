import { Component } from 'react'

interface ErrorBoundaryProps {
  children: React.ReactNode
  onError: (error: Error) => void
  fallback: React.ReactNode
}

export class ErrorBoundaryBasic extends Component<ErrorBoundaryProps, { hasError: boolean }> {
  state = { hasError: false }

  static getDerivedStateFromError() {
    return { hasError: true }
  }

  componentDidCatch(error: Error) {
    this.props.onError(error)
  }

  render() {
    if (this.state.hasError) return this.props.fallback
    return this.props.children
  }
}
