<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { ArrowRight, RefreshCw } from 'lucide-vue-next'
import { RouterLink } from 'vue-router'
import { api, errorMessage, type Job } from '../api'
import { useStatus } from '../stores/status'

const { state, refresh: refreshStatus } = useStatus()
const failures = ref<Job[]>([])
const loading = ref(false)
const error = ref('')
let timer: number | undefined

async function refresh() {
  loading.value = true
  try {
    const result = await api.jobs({ state: 'failed', limit: 5 })
    failures.value = result.jobs
    error.value = ''
    await refreshStatus()
  } catch (reason) {
    error.value = errorMessage(reason)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  void refresh()
  timer = window.setInterval(() => {
    if (document.visibilityState === 'visible') void refresh()
  }, 2000)
})
onBeforeUnmount(() => { if (timer !== undefined) window.clearInterval(timer) })

function stateLabel(value: string) { return value.replace('_', ' ') }
function dateLabel(value: string) { return new Date(value).toLocaleString() }
</script>

<template>
  <div class="page-header">
    <div><p class="eyebrow">Operations</p><h1>Dashboard</h1><p class="page-subtitle">Queue health and recent activity from the local daemon.</p></div>
    <button class="button secondary" type="button" :disabled="loading" @click="refresh"><RefreshCw :size="16" :class="{ spinning: loading }" aria-hidden="true" />Refresh</button>
  </div>

  <p v-if="error" class="alert error" role="alert">{{ error }}</p>
  <section class="metric-grid" aria-label="Queue summary">
    <article class="metric"><span>Worker</span><strong>{{ state.status?.worker_state ?? 'unknown' }}</strong><small>{{ state.status?.current_job_id ? `Job #${state.status.current_job_id}` : 'No active job' }}</small></article>
    <article class="metric"><span>Queued</span><strong>{{ state.status?.queue_counts.queued ?? '-' }}</strong><small>Waiting to run</small></article>
    <article class="metric"><span>Running</span><strong>{{ state.status?.queue_counts.running ?? '-' }}</strong><small>Single worker</small></article>
    <article class="metric"><span>Completed</span><strong>{{ state.status?.queue_counts.succeeded ?? '-' }}</strong><small>{{ state.status?.queue_counts.failed ?? 0 }} failed</small></article>
  </section>

  <section class="section-block" aria-labelledby="current-job-heading">
    <div class="section-heading"><div><p class="eyebrow">Live state</p><h2 id="current-job-heading">Current job</h2></div></div>
    <div v-if="state.status?.current_job_id" class="current-job">
      <div><span class="label">Job ID</span><RouterLink :to="`/jobs/${state.status.current_job_id}`">#{{ state.status.current_job_id }}</RouterLink></div>
      <div><span class="label">Worker state</span><strong>{{ state.status.worker_state }}</strong></div>
      <div><span class="label">Uptime</span><strong>{{ Math.floor((state.status.uptime_seconds ?? 0) / 60) }}m</strong></div>
    </div>
    <p v-else class="empty-state">The worker is idle. Jobs submitted through the API will appear here.</p>
  </section>

  <section class="section-block" aria-labelledby="failures-heading">
    <div class="section-heading"><div><p class="eyebrow">Needs attention</p><h2 id="failures-heading">Recent failures</h2></div><RouterLink class="text-link" to="/jobs?state=failed">View all <ArrowRight :size="15" aria-hidden="true" /></RouterLink></div>
    <div v-if="failures.length" class="table-wrap"><table><thead><tr><th>Job</th><th>Type</th><th>Failed</th><th>Error</th></tr></thead><tbody><tr v-for="job in failures" :key="job.id"><td><RouterLink :to="`/jobs/${job.id}`">#{{ job.id }}</RouterLink></td><td>{{ job.kind }}</td><td><span class="state failed">{{ stateLabel(job.state) }}</span></td><td class="error-cell">{{ job.error || 'No error message' }}</td></tr></tbody></table></div>
    <p v-else class="empty-state">No failed jobs in recent history.</p>
  </section>
</template>
