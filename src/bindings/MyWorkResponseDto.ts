import type { MyWorkItemDto } from './MyWorkItemDto'

export interface MyWorkResponseDto {
  assigned: MyWorkItemDto[]
  mentioned: MyWorkItemDto[]
}