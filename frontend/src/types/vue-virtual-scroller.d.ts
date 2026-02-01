declare module 'vue-virtual-scroller' {
  import { DefineComponent } from 'vue'

  export interface ScrollerMethods {
    scrollToBottom(): void
    scrollToItem(index: number): void
    scrollToPosition(position: number): void
  }

  export const RecycleScroller: DefineComponent<{
    items: any[]
    itemSize: number
    keyField?: string
  }> & ScrollerMethods

  export const DynamicScroller: DefineComponent<{
    items: any[]
    minItemSize: number
    keyField?: string
  }> & ScrollerMethods

  export const DynamicScrollerItem: DefineComponent<{
    item: any
    active: boolean
    sizeDependencies?: any[]
    dataIndex?: number
  }>
}
