import { onBeforeUnmount, onMounted, reactive, readonly } from 'vue'
import { api, errorMessage, type Capabilities, type Status } from '../api'

const state = reactive<{ health: { status: string; version: string } | null; status: Status | null; capabilities: Capabilities | null; error: string | null }>({
  health: null, status: null, capabilities: null, error: null,
})
let timer: number | undefined
let users = 0
let capabilitiesPromise: Promise<Capabilities> | null = null

export function loadCapabilities(): Promise<Capabilities> {
  if (state.capabilities) return Promise.resolve(state.capabilities)
  if (!capabilitiesPromise) {
    capabilitiesPromise = api.capabilities()
      .then((capabilities) => {
        state.capabilities = capabilities
        return capabilities
      })
      .finally(() => { capabilitiesPromise = null })
  }
  return capabilitiesPromise
}

async function refresh() {
  if (document.visibilityState !== 'visible') return
  try {
    const [health, status] = await Promise.all([api.health(), api.status(), loadCapabilities()])
    state.health = health; state.status = status; state.error = null
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
