import { ActivityIcon, GitBranchPlusIcon, MessageSquareIcon } from 'lucide-react'
import { useLocation } from '@tanstack/react-router'
import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { Card, CardContent } from '@/components/ui/card'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { useIssueViewmodel } from '../issue.viewmodel'
import { IssueActivity } from './issue-activity'
import { IssueAddComment } from './issue-add-comment'
import { IssueAttachments } from './issue-attachments'
import { IssueChildren } from './issue-children'
import { IssueComments } from './issue-comments'

export const IssueHistory = observer(() => {
  const viewmodel = useIssueViewmodel()
  const location = useLocation()
  const [activeTab, setActiveTab] = useState('comments')

  const { issue } = viewmodel

  if (!issue) return null

  useEffect(() => {
    if (location.hash.startsWith('#comment-')) {
      setActiveTab('comments')
    }
  }, [location.hash])

  return (
    <Card className="overflow-visible">
      <CardContent>
        <Tabs value={activeTab} onValueChange={setActiveTab}>
          <TabsList variant="line">
            <TabsTrigger value="comments">
              <MessageSquareIcon className="size-4" />
              Comments
            </TabsTrigger>
            <TabsTrigger value="children">
              <GitBranchPlusIcon className="size-4" />
              Children
            </TabsTrigger>
            <TabsTrigger value="activity">
              <ActivityIcon className="size-4" />
              Activity
            </TabsTrigger>
          </TabsList>

          <TabsContent className="mt-5 space-y-4" value="comments">
            <IssueAddComment />
            <IssueAttachments />
            <IssueComments />
          </TabsContent>

          <TabsContent className="mt-5" value="children">
            <IssueChildren />
          </TabsContent>

          <TabsContent className="mt-5" value="activity">
            <IssueActivity />
          </TabsContent>
        </Tabs>
      </CardContent>
    </Card>
  )
})
