import type { DndContextProps, DragEndEvent, DragOverEvent, DragStartEvent } from '@dnd-kit/core'
import {
  closestCenter,
  DndContext,
  DragOverlay,
  KeyboardSensor,
  MouseSensor,
  TouchSensor,
  useDroppable,
  useSensor,
  useSensors,
} from '@dnd-kit/core'
import { arrayMove, SortableContext, useSortable } from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { createContext, type HTMLAttributes, type ReactNode, useContext, useState } from 'react'
import { createPortal } from 'react-dom'
import tunnelRat from 'tunnel-rat'
import { Card } from '@/components/atoms/card'
import { cn } from '@/lib/utils/cn'

const tunnel = tunnelRat()

export type { DragEndEvent } from '@dnd-kit/core'

type KanbanItemProps = {
  id: string
  name: string
  column: string
} & Record<string, unknown>

type KanbanColumnProps = {
  id: string
  name: string
} & Record<string, unknown>

type KanbanContextProps<
  T extends KanbanItemProps = KanbanItemProps,
  C extends KanbanColumnProps = KanbanColumnProps,
> = {
  columns: C[]
  data: T[]
  activeCardId: string | null
}

const KanbanContext = createContext<KanbanContextProps>({
  activeCardId: null,
  columns: [],
  data: [],
})

export type KanbanBoardProps = {
  id: string
  children: ReactNode
  className?: string
}

export const KanbanBoard = ({ id, children, className }: KanbanBoardProps) => {
  const { isOver, setNodeRef } = useDroppable({ id })

  return (
    <div
      ref={setNodeRef}
      className={cn(
        'flex size-full min-h-40 flex-col divide-y divide-border/60 overflow-hidden rounded-lg border border-border/60 bg-background/40 text-xs shadow-none transition-all',
        isOver && 'brightness-150 ring-1 ring-primary/30',
        className,
      )}
    >
      {children}
    </div>
  )
}

export type KanbanCardProps<T extends KanbanItemProps = KanbanItemProps> = T & {
  children?: ReactNode
  className?: string
}

export const KanbanCard = <T extends KanbanItemProps = KanbanItemProps>({
  id,
  name,
  children,
  className,
}: KanbanCardProps<T>) => {
  const { attributes, setNodeRef, transition, transform, isDragging } = useSortable({
    id,
  })
  const { activeCardId } = useContext(KanbanContext) as KanbanContextProps

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  }

  return (
    <>
      <div style={style} {...attributes} ref={setNodeRef}>
        <Card
          className={cn(
            'gap-3 rounded-md border border-border/60 bg-background/60 p-2 shadow-none',
            isDragging && 'pointer-events-none cursor-grabbing opacity-30',
            className,
          )}
        >
          {children ?? <p className="m-0 font-medium">{name}</p>}
        </Card>
      </div>
      {activeCardId === id && (
        <tunnel.In>
          <Card
            className={cn(
              'gap-2 rounded-md border border-primary/50 bg-background/40 p-2 shadow-none backdrop-blur-sm',
              isDragging && 'cursor-grabbing',
              className,
            )}
          >
            {children ?? <p className="m-0 font-medium">{name}</p>}
          </Card>
        </tunnel.In>
      )}
    </>
  )
}

type KanbanSortableHandleProps = React.PropsWithChildren<{
  id: string
}>

export const KanbanSortableHandle = ({ id, children }: KanbanSortableHandleProps) => {
  const { attributes, listeners, setNodeRef, transition, transform } = useSortable({
    id,
  })

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  }

  return (
    <div className="cursor-grab" style={style} {...listeners} {...attributes} ref={setNodeRef}>
      {children}
    </div>
  )
}

export const KanbanCardWithHandle = <T extends KanbanItemProps = KanbanItemProps>({
  id,
  name,
  children,
  className,
}: KanbanCardProps<T>) => {
  const { attributes, listeners, setActivatorNodeRef, setNodeRef, transition, transform, isDragging } = useSortable({
    id,
  })
  const { activeCardId } = useContext(KanbanContext) as KanbanContextProps
  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  }
  return (
    <>
      <div ref={setNodeRef} style={style}>
        <Card
          className={cn(
            'px-1 pt-1 pb-0 border-0 bg-transparent hover:bg-transparent',
            isDragging && 'pointer-events-none cursor-grabbing opacity-30',
            className,
          )}
        >
          <div className="flex items-center justify-center pb-1">
            <div
              ref={setActivatorNodeRef}
              className="flex h-2 w-7 cursor-grab items-center justify-center rounded-full bg-border/70"
              {...listeners}
              {...attributes}
            />
          </div>
          {children ?? <p className="m-0 font-medium text-sm">{name}</p>}
        </Card>
      </div>
      {activeCardId === id && (
        <tunnel.In>
          <Card
            className={cn(
              'px-1 pt-1 pb-0 border-0 bg-transparent hover:bg-transparent backdrop-blur-none',
              isDragging && 'cursor-grabbing',
              className,
            )}
          >
            <div className="flex items-center justify-center pb-1">
              <div className="flex h-2 w-7 cursor-grab items-center justify-center rounded-full bg-border/70" />
            </div>
            {children ?? <p className="m-0 font-medium text-sm">{name}</p>}
          </Card>
        </tunnel.In>
      )}
    </>
  )
}

export type KanbanCardsProps<T extends KanbanItemProps = KanbanItemProps> = Omit<
  HTMLAttributes<HTMLDivElement>,
  'children' | 'id'
> & {
  children: (item: T) => ReactNode
  id: string
  items?: T[]
}

export const KanbanCards = <T extends KanbanItemProps = KanbanItemProps>({
  children,
  className,
  items,
  ...props
}: KanbanCardsProps<T>) => {
  const { data } = useContext(KanbanContext) as KanbanContextProps<T>
  const sourceData = items ?? data
  const filteredData = sourceData.filter((item) => item.column === props.id)
  const itemIds = filteredData.map((item) => item.id)

  return (
    <div className="flex h-full flex-col overflow-y-auto">
      <SortableContext items={itemIds}>
        <div className={cn('flex grow flex-col gap-2 px-2 pb-2 text-xs', className)} {...props}>
          {filteredData.length ? (
            filteredData.map(children)
          ) : (
            <div className="flex min-h-24 items-center justify-center rounded-md border border-dashed border-border/70 bg-muted/20 px-3 py-6 text-[11px] text-muted-foreground/70 mt-2">
              No items
            </div>
          )}
        </div>
      </SortableContext>
    </div>
  )
}

export type KanbanHeaderProps = HTMLAttributes<HTMLDivElement>

export const KanbanHeader = ({ className, ...props }: KanbanHeaderProps) => (
  <div
    className={cn('m-0 flex items-center px-3 py-2 text-[11px] font-semibold text-muted-foreground', className)}
    {...props}
  />
)

export type KanbanProviderProps<
  T extends KanbanItemProps = KanbanItemProps,
  C extends KanbanColumnProps = KanbanColumnProps,
> = Omit<DndContextProps, 'children'> & {
  children: (column: C) => ReactNode
  className?: string
  columns: C[]
  data: T[]
  onDataChange?: (data: T[]) => void
  onDragStart?: (event: DragStartEvent) => void
  onDragEnd?: (event: DragEndEvent) => void
  onDragOver?: (event: DragOverEvent) => void
}

export const KanbanProvider = <
  T extends KanbanItemProps = KanbanItemProps,
  C extends KanbanColumnProps = KanbanColumnProps,
>({
  children,
  onDragStart,
  onDragEnd,
  onDragOver,
  className,
  columns,
  data,
  onDataChange,
  ...props
}: KanbanProviderProps<T, C>) => {
  const [activeCardId, setActiveCardId] = useState<string | null>(null)

  const sensors = useSensors(useSensor(MouseSensor), useSensor(TouchSensor), useSensor(KeyboardSensor))

  const handleDragStart = (event: DragStartEvent) => {
    const card = data.find((item) => item.id === event.active.id)
    if (card) {
      setActiveCardId(event.active.id as string)
    }
    onDragStart?.(event)
  }

  const handleDragOver = (event: DragOverEvent) => {
    const { active, over } = event

    if (!over) {
      return
    }

    const activeItem = data.find((item) => item.id === active.id)
    const overItem = data.find((item) => item.id === over.id)

    if (!activeItem) {
      return
    }

    const activeColumn = activeItem.column
    const overColumn = overItem?.column || columns.find((col) => col.id === over.id)?.id || columns[0]?.id

    if (activeColumn !== overColumn) {
      let newData = [...data]
      const activeIndex = newData.findIndex((item) => item.id === active.id)
      const overIndex = newData.findIndex((item) => item.id === over.id)

      if (activeIndex === -1) {
        return
      }

      newData[activeIndex].column = overColumn

      if (overIndex !== -1) {
        newData = arrayMove(newData, activeIndex, overIndex)
      }

      onDataChange?.(newData)
    }

    onDragOver?.(event)
  }

  const handleDragEnd = (event: DragEndEvent) => {
    setActiveCardId(null)

    onDragEnd?.(event)

    const { active, over } = event

    if (!over || active.id === over.id) {
      return
    }

    let newData = [...data]

    const oldIndex = newData.findIndex((item) => item.id === active.id)
    const newIndex = newData.findIndex((item) => item.id === over.id)

    if (oldIndex === -1 || newIndex === -1) {
      return
    }

    newData = arrayMove(newData, oldIndex, newIndex)

    onDataChange?.(newData)
  }

  return (
    <KanbanContext.Provider value={{ activeCardId, columns, data }}>
      <DndContext
        collisionDetection={closestCenter}
        sensors={sensors}
        onDragEnd={handleDragEnd}
        onDragOver={handleDragOver}
        onDragStart={handleDragStart}
        {...props}
      >
        <div className="h-full w-full overflow-x-auto">
          <div className="flex h-full min-w-6xl justify-center">
            <div className={cn('grid size-full auto-cols-fr grid-flow-col gap-3 w-6xl pb-3', className)}>
              {columns.map((column) => children(column))}
            </div>
          </div>
        </div>
        {typeof window !== 'undefined' &&
          createPortal(
            <DragOverlay>
              <tunnel.Out />
            </DragOverlay>,
            document.body,
          )}
      </DndContext>
    </KanbanContext.Provider>
  )
}
