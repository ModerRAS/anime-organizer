<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { ChevronDown, CircleSlash, RotateCcw } from 'lucide-vue-next'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import { api, errorMessage, type Job, type JobState } from '../api'
import { jobQueryState, jobStates } from '../filters'

const route = useRoute()
const router = useRouter()
const jobs = ref<Job[]>([])
const loading = ref(false)
const error = ref('')
const actionError = ref('')
const selectedState = ref<JobState | ''>(jobQueryState(route.query.state) ?? '')
const kind = ref(typeof route.query.kind === 'string' ? route.query.kind : '')
let timer: number | undefined

async function load() {
  loading.value = true
  try {
    const result = await api.jobs({ state: selectedState.value || undefined, kind: kind.value || undefined, limit: 100 })
    jobs.value = result.jobs
    error.value = ''
  } catch (reason) { error.value = errorMessage(reason) }
  finally { loading.value = false }
}

function updateQuery() {
  void router.replace({ query: { ...(selectedState.value ? { state: selectedState.value } : {}), ...(kind.value ? { kind: kind.value } : {}) } })
  void load()
}
async function cancel(job: Job) {
  actionError.value = ''
  try { await api.cancel(job.id); await load() } catch (reason) { actionError.value = errorMessage(reason) }
}
async function retry(job: Job) {
  actionError.value = ''
  try { await api.retry(job.id); await load() } catch (reason) { actionError.value = errorMessage(reason) }
}
function stateLabel(value: string) { return value.replace('_', ' ') }
function dateLabel(value: string) { return new Date(value).toLocaleString() }
function statusClass(value: JobState) { return value }

watch(() => route.query, () => {
  selectedState.value = jobQueryState(route.query.state) ?? ''
  kind.value = typeof route.query.kind === 'string' ? route.query.kind : ''
  void load()
}, { deep: true })
onMounted(() => {
  void load()
  timer = window.setInterval(() => { if (document.visibilityState === 'visible') void load() }, 2000)
})
onBeforeUnmount(() => { if (timer !== undefined) window.clearInterval(timer) })
</script>

<template>
  <div class="page-header"><div><p class="eyebrow">Queue</p><h1>Jobs</h1><p class="page-subtitle">Inspect accepted work and control jobs that have not started.</p></div><span class="record-count">{{ jobs.length }} records</span></div>
  <p v-if="error" class="alert error" role="alert">{{ error }}</p>
  <p v-if="actionError" class="alert error" role="alert">{{ actionError }}</p>
  <section class="section-block jobs-panel" aria-labelledby="jobs-table-heading">
    <div class="filter-bar" aria-label="Job filters">
      <label>State<select v-model="selectedState" @change="updateQuery"><option v-for="item in jobStates" :key="item.value" :value="item.value">{{ item.label }}</option></select></label>
      <label>Type<input v-model="kind" type="search" placeholder="All types" @change="updateQuery" /></label>
      <button class="button secondary filter-reset" type="button" :disabled="!selectedState && !kind" @click="selectedState = ''; kind = ''; updateQuery()"><ChevronDown :size="15" aria-hidden="true" />Clear filters</button>
    </div>
    <div class="table-wrap"><table><caption id="jobs-table-heading" class="sr-only">Daemon jobs</caption><thead><tr><th>Job</th><th>Type</th><th>Origin</th><th>State</th><th>Created</th><th><span class="sr-only">Actions</span></th></tr></thead><tbody>
      <tr v-for="job in jobs" :key="job.id"><td><RouterLink :to="`/jobs/${job.id}`">#{{ job.id }}</RouterLink><small v-if="job.progress_message" class="table-subtext">{{ job.progress_message }}</small></td><td>{{ job.kind }}</td><td>{{ job.origin }}</td><td><span class="state" :class="statusClass(job.state)">{{ stateLabel(job.state) }}</span></td><td>{{ dateLabel(job.created_at) }}</td><td class="actions"><button v-if="job.state === 'queued'" class="icon-button danger-action" type="button" title="Cancel queued job" aria-label="Cancel queued job" @click="cancel(job)"><CircleSlash :size="16" aria-hidden="true" /></button><button v-if="job.state === 'failed' || job.state === 'canceled'" class="icon-button" type="button" title="Retry job" aria-label="Retry job" @click="retry(job)"><RotateCcw :size="16" aria-hidden="true" /></button></td></tr>
      <tr v-if="!loading && !jobs.length"><td colspan="6" class="empty-cell">No jobs match these filters.</td></tr>
    </tbody></table></div>
    <p v-if="loading" class="loading-line">Refreshing queue...</p>
  </section>
</template>
