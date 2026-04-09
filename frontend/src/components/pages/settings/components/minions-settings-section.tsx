import { BotIcon, PlusIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { MinionSheet } from '@/components/forms/minion/minion-sheet'
import { AppLoader } from '@/components/organisms/app-loader'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { MinionsViewmodel, MinionsViewmodelContext } from '../minions.viewmodel'
import { MinionCard } from './minion-card'

export const MinionsSettingsSection = observer(() => {
  const [viewmodel] = useState(() => new MinionsViewmodel())

  useEffect(() => {
    void viewmodel.init()
  }, [viewmodel])

  if (viewmodel.isLoading) return <AppLoader />

  return (
    <MinionsViewmodelContext.Provider value={viewmodel}>
      <div className="flex flex-col gap-4">
        {viewmodel.errorMessage ? (
          <Card>
            <CardContent className="py-4 text-sm text-destructive">{viewmodel.errorMessage}</CardContent>
          </Card>
        ) : null}

        <Card>
          <CardContent className="flex flex-col gap-4 py-4 md:flex-row md:items-center md:justify-between">
            <div className="space-y-1">
              <div className="flex items-center gap-2 font-medium">
                <BotIcon className="size-4" />
                Minions
              </div>
              <p className="text-sm text-muted-foreground">
                Manage built-in system minions and any custom owner-defined minions.
              </p>
            </div>

            <Button type="button" onClick={viewmodel.openCreate}>
              <PlusIcon className="size-4" />
              New minion
            </Button>
          </CardContent>
        </Card>

        {viewmodel.minions.length === 0 ? (
          <Card>
            <CardContent className="py-8 text-sm text-muted-foreground">No minions available yet.</CardContent>
          </Card>
        ) : (
          <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
            {viewmodel.minions.map((minion) => (
              <MinionCard
                key={minion.id}
                isDeleting={viewmodel.isDeletingMinionId === minion.id}
                minion={minion}
                onDelete={() => viewmodel.deleteMinion(minion.id)}
                onOpen={() => viewmodel.openEdit(minion)}
              />
            ))}
          </div>
        )}

        <MinionSheet viewmodel={viewmodel.sheet} />
      </div>
    </MinionsViewmodelContext.Provider>
  )
})
