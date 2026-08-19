import { onUnmounted, shallowRef } from 'vue'

const MOVE_THRESHOLD = 4

export function reorderIdFromPoint(clientX: number, clientY: number) {
  const node = document.elementFromPoint(clientX, clientY)
  return node?.closest('[data-reorder-id]')?.getAttribute('data-reorder-id') ?? null
}

export function usePointerReorder(
  onReorder: (fromId: string, toId: string) => void,
  canDrop?: (fromId: string, toId: string) => boolean,
) {
  const dragId = shallowRef<string | null>(null)
  const overId = shallowRef<string | null>(null)

  let pointerId: number | null = null
  let started = false
  let startX = 0
  let startY = 0
  let handle: HTMLElement | null = null

  function allowed(fromId: string, toId: string) {
    return !canDrop || canDrop(fromId, toId)
  }

  function stop(event?: PointerEvent) {
    if (handle && event && pointerId != null) {
      try {
        handle.releasePointerCapture(pointerId)
      } catch {
        // capture may already be released
      }
    }
    window.removeEventListener('pointermove', onMove, true)
    window.removeEventListener('pointerup', onUp, true)
    window.removeEventListener('pointercancel', onUp, true)
    document.body.classList.remove('is-reordering')
    pointerId = null
    started = false
    handle = null
  }

  function onMove(event: PointerEvent) {
    if (event.pointerId !== pointerId || !dragId.value) return
    if (!started) {
      const dx = event.clientX - startX
      const dy = event.clientY - startY
      if (dx * dx + dy * dy < MOVE_THRESHOLD * MOVE_THRESHOLD) return
      started = true
      document.body.classList.add('is-reordering')
    }
    const over = reorderIdFromPoint(event.clientX, event.clientY)
    if (over && allowed(dragId.value, over)) overId.value = over
  }

  function onUp(event: PointerEvent) {
    if (event.pointerId !== pointerId) return
    const from = dragId.value
    const to = overId.value
    const didDrag = started
    stop(event)
    dragId.value = null
    overId.value = null
    if (didDrag && from && to && from !== to && allowed(from, to)) onReorder(from, to)
  }

  function onHandlePointerDown(id: string, event: PointerEvent) {
    if (event.button !== 0) return
    event.preventDefault()
    handle = event.currentTarget as HTMLElement
    pointerId = event.pointerId
    started = false
    startX = event.clientX
    startY = event.clientY
    dragId.value = id
    overId.value = id
    handle.setPointerCapture(event.pointerId)
    window.addEventListener('pointermove', onMove, true)
    window.addEventListener('pointerup', onUp, true)
    window.addEventListener('pointercancel', onUp, true)
  }

  onUnmounted(() => {
    stop()
    dragId.value = null
    overId.value = null
  })

  return { dragId, overId, onHandlePointerDown }
}
