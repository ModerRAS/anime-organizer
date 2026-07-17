import { onBeforeUnmount, onMounted, reactive, readonly } from 'vue'
import { api, errorMessage, type Capabilities, type Status } from '../api'

const state = reactive<{ health: { status: string; version: string } | null; status: Status | null; capabilities: Capabilities | null; error: string | null }>({
  health: null, status: null, capabilities: null, error: null,
})
let timer: number | undefined
let users = 0

async function refresh() {
  if (document.visibilityState !== 'visible') return
  try {
    const [health, status, capabilities] = await Promise.all([api.health(), api.status(), api.capabilities()])
    state.health = health; state.status = status; state.capabilities = capabilities; state.error = null
  } catch (error) { state.error = errorMessage(error) }
}

function start() {
  users += 1
  if (users === 1) { void refresh(); timer = window.setInterval(refresh, 2000) }
}
function stop() {
  users -= 1
  if (users <= 0) { users = 0; if (timer !== undefined) window.clearInterval(timer); timer = undefined }
}

export function useStatus() {
  onMounted(start)
  onBeforeUnmount(stop)
  return { state: readonly(state), refresh }
}
