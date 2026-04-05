import { useLocation } from '@tanstack/react-router'
import { observer } from 'mobx-react-lite'
import { useEffect, useRef } from 'react'
import { Page } from '@/components/layouts/page'
import { ScrollToBottomButton } from '@/components/molecules/scroll-to-bottom-button'
import { useScrollAnchor } from '@/hooks/use-scroll-anchor'
import { IssueDetails } from './components/issue-details'
import { IssueHistory } from './components/issue-history'
import { IssueMetadata } from './components/issue-metadata'
import { IssueNotFound } from './components/issue-not-found'
import { useIssueViewmodel } from './issue.viewmodel'

export const IssuePage = observer(() => {
  const viewmodel = useIssueViewmodel()
  const location = useLocation()
  const pageRef = useRef<HTMLDivElement | null>(null)
  const scrollAnchor = useScrollAnchor()

  useEffect(() => {
    if (!viewmodel.issue || !location.hash.startsWith('#comment-')) return

    const commentId = location.hash.slice(1)
    const scrollToComment = () => {
      const target = document.getElementById(commentId)
      if (!target) return
      target.scrollIntoView({ behavior: 'smooth', block: 'center' })
      scrollAnchor.updateScrollState()
    }

    const frame = window.requestAnimationFrame(scrollToComment)
    return () => window.cancelAnimationFrame(frame)
  }, [location.hash, scrollAnchor, viewmodel.issue])

  if (!viewmodel.issue) return <IssueNotFound />

  return (
    <>
      <Page
        ref={(element) => {
          pageRef.current = element
          scrollAnchor.setContainer(element)
        }}
        className="overflow-y-auto p-1 pr-2"
      >
        <div className="flex gap-3 flex-col lg:flex-row lg:justify-between">
        <div className="flex min-w-0 flex-col gap-3 max-w-5xl">
          <IssueDetails />

          <IssueHistory />
        </div>

        <div className="w-full lg:w-[240px] shrink-0">
          <IssueMetadata />
        </div>
      </div>
      </Page>
      <ScrollToBottomButton onClick={() => scrollAnchor.scrollToBottom()} visible={!scrollAnchor.isNearBottom} />
    </>
  )
})
