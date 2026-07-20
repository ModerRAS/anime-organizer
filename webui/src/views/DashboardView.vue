<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { ArrowRight, RefreshCw } from 'lucide-vue-next'
import { RouterLink } from 'vue-router'
import { api, errorMessage, type Job } from '../api'
import { t, valueLabel } from '../i18n'
import { useStatus } from '../stores/status'

const { state } = useStatus()
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

</script>

<template>
  <div class="page-header">
    <div><p class="eyebrow">{{ t('Operations') }}</p><h1>{{ t('Dashboard') }}</h1><p class="page-subtitle">{{ t('Queue health and recent activity from the local daemon.') }}</p></div>
    <button class="button secondary" type="button" :disabled="loading" @click="refresh"><RefreshCw :size="16" :class="{ spinning: loading }" aria-hidden="true" />{{ t('Refresh') }}</button>
  </div>

  <p v-if="error" class="alert error" role="alert">{{ error }}</p>
  <section class="metric-grid" :aria-label="t('Queue summary')">
    <article class="metric"><span>{{ t('Worker') }}</span><strong>{{ valueLabel(state.status?.worker_state ?? 'unknown') }}</strong><small>{{ state.status?.current_job_id ? `${t('Job')} #${state.status.current_job_id}` : t('No active job') }}</small></article>
    <article class="metric"><span>{{ t('Queued') }}</span><strong>{{ state.status?.queue_counts.queued ?? '-' }}</strong><small>{{ t('Waiting to run') }}</small></article>
    <article class="metric"><span>{{ t('Running') }}</span><strong>{{ state.status?.queue_counts.running ?? '-' }}</strong><small>{{ t('Single worker') }}</small></article>
    <article class="metric"><span>{{ t('Completed') }}</span><strong>{{ state.status?.queue_counts.succeeded ?? '-' }}</strong><small>{{ t('{count} failed', { count: state.status?.queue_counts.failed ?? 0 }) }}</small></article>
  </section>

  <section class="section-block" aria-labelledby="current-job-heading">
    <div class="section-heading"><div><p class="eyebrow">{{ t('Live state') }}</p><h2 id="current-job-heading">{{ t('Current job') }}</h2></div></div>
    <div v-if="state.status?.current_job_id" class="current-job">
      <div><span class="label">{{ t('Job ID') }}</span><RouterLink :to="`/jobs/${state.status.current_job_id}`">#{{ state.status.current_job_id }}</RouterLink></div>
      <div><span class="label">{{ t('Worker state') }}</span><strong>{{ valueLabel(state.status.worker_state) }}</strong></div>
      <div><span class="label">{{ t('Uptime') }}</span><strong>{{ t('{minutes}m', { minutes: Math.floor((state.status.uptime_seconds ?? 0) / 60) }) }}</strong></div>
    </div>
    <p v-else class="empty-state">{{ t('The worker is idle. Jobs submitted through the API will appear here.') }}</p>
  </section>

  <section class="section-block" aria-labelledby="failures-heading">
    <div class="section-heading"><div><p class="eyebrow">{{ t('Needs attention') }}</p><h2 id="failures-heading">{{ t('Recent failures') }}</h2></div><RouterLink class="text-link" to="/jobs?state=failed">{{ t('View all') }} <ArrowRight :size="15" aria-hidden="true" /></RouterLink></div>
    <div v-if="failures.length" class="table-wrap"><table><thead><tr><th>{{ t('Job') }}</th><th>{{ t('Type') }}</th><th>{{ t('State') }}</th><th>{{ t('Error') }}</th></tr></thead><tbody><tr v-for="job in failures" :key="job.id"><td><RouterLink :to="`/jobs/${job.id}`">#{{ job.id }}</RouterLink></td><td>{{ valueLabel(job.kind) }}</td><td><span class="state failed">{{ valueLabel(job.state) }}</span></td><td class="error-cell">{{ job.error || t('No error message') }}</td></tr></tbody></table></div>
    <p v-else class="empty-state">{{ t('No failed jobs in recent history.') }}</p>
  </section>
</template>
