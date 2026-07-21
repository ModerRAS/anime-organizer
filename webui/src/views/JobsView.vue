<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { ChevronDown, CircleSlash, RotateCcw } from 'lucide-vue-next'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import { api, errorMessage, type Job, type JobState } from '../api'
import { jobQueryState, jobStates } from '../filters'
import { formatDateTime, formatDuration, t, valueLabel } from '../i18n'

const pageSize = 100
const route = useRoute()
const router = useRouter()
const jobs = ref<Job[]>([])
const loading = ref(false)
const loadingOlder = ref(false)
const hasOlder = ref(false)
const error = ref('')
const actionError = ref('')
const selectedState = ref<JobState | ''>(jobQueryState(route.query.state) ?? '')
const kind = ref(typeof route.query.kind === 'string' ? route.query.kind : '')
let timer: number | undefined
let queryVersion = 0
let refreshVersion = 0

function query(beforeId?: number) {
  return { state: selectedState.value || undefined, kind: kind.value || undefined, limit: pageSize, before_id: beforeId }
}

async function load(reset = false) {
  const version = queryVersion
  const refresh = ++refreshVersion
  loading.value = true
  try {
    const result = await api.jobs(query())
    if (version !== queryVersion || refresh !== refreshVersion) return
    const latestIds = new Set(result.jobs.map(job => job.id))
    const restartPagination = !reset && jobs.value.length > 0 && result.jobs.length === pageSize && !jobs.value.some(job => latestIds.has(job.id))
    jobs.value = reset || restartPagination ? result.jobs : [...result.jobs, ...jobs.value.filter(job => !latestIds.has(job.id))]
    if (reset || restartPagination) hasOlder.value = result.jobs.length === pageSize
    error.value = ''
  } catch (reason) {
    if (version === queryVersion && refresh === refreshVersion) error.value = errorMessage(reason)
  } finally {
    if (version === queryVersion && refresh === refreshVersion) loading.value = false
  }
}

async function loadOlder() {
  const beforeId = jobs.value[jobs.value.length - 1]?.id
  if (beforeId === undefined) return
  const version = queryVersion
  loadingOlder.value = true
  try {
    const result = await api.jobs(query(beforeId))
    if (version !== queryVersion) return
    jobs.value = [...new Map([...jobs.value, ...result.jobs].map((job) => [job.id, job])).values()]
    hasOlder.value = result.jobs.length === pageSize
    error.value = ''
  } catch (reason) {
    if (version === queryVersion) error.value = errorMessage(reason)
  } finally {
    if (version === queryVersion) loadingOlder.value = false
  }
}

function updateQuery() {
  void router.replace({ query: { ...(selectedState.value ? { state: selectedState.value } : {}), ...(kind.value ? { kind: kind.value } : {}) } })
}
async function cancel(job: Job) {
  if (!window.confirm(t('Cancel this queued job?'))) return
  actionError.value = ''
  try { await api.cancel(job.id); await load() } catch (reason) { actionError.value = errorMessage(reason) }
}
async function retry(job: Job) {
  actionError.value = ''
  try { await api.retry(job.id); await load() } catch (reason) { actionError.value = errorMessage(reason) }
}
function statusClass(value: JobState) { return value }

watch(() => route.query, () => {
  queryVersion++
  selectedState.value = jobQueryState(route.query.state) ?? ''
  kind.value = typeof route.query.kind === 'string' ? route.query.kind : ''
  jobs.value = []
  hasOlder.value = false
  loadingOlder.value = false
  error.value = ''
  actionError.value = ''
  void load(true)
}, { deep: true })
onMounted(() => {
  void load(true)
  timer = window.setInterval(() => { if (document.visibilityState === 'visible') void load() }, 2000)
})
onBeforeUnmount(() => { if (timer !== undefined) window.clearInterval(timer) })
</script>

<template>
  <div class="page-header"><div><p class="eyebrow">{{ t('Queue') }}</p><h1>{{ t('Jobs') }}</h1><p class="page-subtitle">{{ t('Inspect accepted work and control jobs that have not started.') }}</p></div><span class="record-count">{{ t('{count} records', { count: jobs.length }) }}</span></div>
  <p v-if="error" class="alert error" role="alert">{{ error }}</p>
  <p v-if="actionError" class="alert error" role="alert">{{ actionError }}</p>
  <section class="section-block jobs-panel" aria-labelledby="jobs-table-heading">
    <div class="filter-bar" :aria-label="t('Job filters')">
      <label>{{ t('State') }}<select v-model="selectedState" @change="updateQuery"><option v-for="item in jobStates" :key="item.value" :value="item.value">{{ t(item.label) }}</option></select></label>
      <label>{{ t('Type') }}<input v-model="kind" type="search" :placeholder="t('All types')" @change="updateQuery" /></label>
      <button class="button secondary filter-reset" type="button" :disabled="!selectedState && !kind" @click="selectedState = ''; kind = ''; updateQuery()"><ChevronDown :size="15" aria-hidden="true" />{{ t('Clear filters') }}</button>
    </div>
    <div class="table-wrap"><table><caption id="jobs-table-heading" class="sr-only">{{ t('Daemon jobs') }}</caption><thead><tr><th>{{ t('Job') }}</th><th>{{ t('Type') }}</th><th>{{ t('Origin') }}</th><th>{{ t('State') }}</th><th>{{ t('Created') }}</th><th>{{ t('Finished') }}</th><th>{{ t('Duration') }}</th><th><span class="sr-only">{{ t('Actions') }}</span></th></tr></thead><tbody>
      <tr v-for="job in jobs" :key="job.id"><td><RouterLink :to="`/jobs/${job.id}`">#{{ job.id }}</RouterLink><small v-if="job.progress_message" class="table-subtext">{{ job.progress_message }}</small></td><td>{{ valueLabel(job.kind) }}</td><td>{{ valueLabel(job.origin) }}</td><td><span class="state" :class="statusClass(job.state)">{{ valueLabel(job.state) }}</span></td><td>{{ formatDateTime(job.created_at) }}</td><td>{{ formatDateTime(job.finished_at) }}</td><td>{{ formatDuration(job.started_at, job.finished_at) }}</td><td class="actions"><button v-if="job.state === 'queued'" class="icon-button danger-action" type="button" :title="t('Cancel queued job')" :aria-label="t('Cancel queued job')" @click="cancel(job)"><CircleSlash :size="16" aria-hidden="true" /></button><button v-if="job.state === 'failed' || job.state === 'canceled'" class="icon-button" type="button" :title="t('Retry job')" :aria-label="t('Retry job')" @click="retry(job)"><RotateCcw :size="16" aria-hidden="true" /></button></td></tr>
      <tr v-if="!loading && !jobs.length"><td colspan="8" class="empty-cell">{{ t('No jobs match these filters.') }}</td></tr>
    </tbody></table></div>
    <p v-if="loading" class="loading-line">{{ t('Refreshing queue...') }}</p>
    <div v-if="hasOlder" class="pagination-actions"><button class="button secondary" type="button" :disabled="loadingOlder" @click="loadOlder">{{ t(loadingOlder ? 'Loading older jobs...' : 'Load older') }}</button></div>
  </section>
</template>
