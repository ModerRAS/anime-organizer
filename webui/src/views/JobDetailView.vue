<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { ArrowLeft, CircleSlash, RotateCcw } from 'lucide-vue-next'
import { RouterLink, useRoute } from 'vue-router'
import { api, errorMessage, type Job, type JobArtifact } from '../api'
import { formatDateTime, t, valueLabel } from '../i18n'

const route = useRoute()
const job = ref<Job | null>(null)
const loading = ref(true)
const error = ref('')
const actionError = ref('')
let timer: number | undefined
let loadVersion = 0

async function load(changedId = false) {
  const version = ++loadVersion
  if (changedId) {
    job.value = null
    error.value = ''
    actionError.value = ''
  }
  const rawId = route.params.id
  const id = typeof rawId === 'string' ? Number(rawId) : NaN
  loading.value = true
  if (!Number.isSafeInteger(id) || id <= 0) {
    error.value = t('Job not found.')
    loading.value = false
    return
  }
  try {
    const result = await api.job(id)
    if (version !== loadVersion) return
    job.value = result
    error.value = ''
  } catch (reason) {
    if (version !== loadVersion) return
    job.value = null
    actionError.value = ''
    error.value = errorMessage(reason)
  } finally {
    if (version === loadVersion) loading.value = false
  }
}
async function cancel() {
  if (!job.value || !window.confirm(t('Cancel this queued job?'))) return
  const version = loadVersion
  actionError.value = ''
  try {
    const result = await api.cancel(job.value.id)
    if (version === loadVersion) job.value = result
  } catch (reason) {
    if (version === loadVersion) actionError.value = errorMessage(reason)
  }
}
async function retry() {
  if (!job.value) return
  const version = loadVersion
  actionError.value = ''
  try {
    const result = await api.retry(job.value.id)
    if (version === loadVersion) job.value = result
  } catch (reason) {
    if (version === loadVersion) actionError.value = errorMessage(reason)
  }
}
function artifactsFor(value: Job): JobArtifact[] {
  const result = value.result
  if (!result || typeof result !== 'object' || !Array.isArray((result as { artifacts?: unknown }).artifacts)) return []
  return (result as { artifacts: JobArtifact[] }).artifacts
}
function isActive() { return job.value?.state === 'queued' || job.value?.state === 'running' }
function startPolling() {
  if (timer !== undefined) window.clearInterval(timer)
  timer = window.setInterval(() => { if (document.visibilityState === 'visible' && isActive()) void load() }, 2000)
}
watch(() => route.params.id, () => { void load(true) })
onMounted(() => { void load(true); startPolling() })
onBeforeUnmount(() => { if (timer !== undefined) window.clearInterval(timer) })
</script>

<template>
  <div class="page-header detail-header"><div><RouterLink class="back-link" to="/jobs"><ArrowLeft :size="15" aria-hidden="true" />{{ t('Back to jobs') }}</RouterLink><p class="eyebrow">{{ t('Job detail') }}</p><h1 v-if="job">#{{ job.id }} <span class="state" :class="job.state">{{ valueLabel(job.state) }}</span></h1><h1 v-else>{{ t('Job') }}</h1></div><div v-if="job" class="detail-actions"><button v-if="job.state === 'queued'" class="button secondary" type="button" :title="t('Only queued jobs can be canceled')" @click="cancel"><CircleSlash :size="16" aria-hidden="true" />{{ t('Cancel') }}</button><button v-if="job.state === 'failed' || job.state === 'canceled'" class="button secondary" type="button" @click="retry"><RotateCcw :size="16" aria-hidden="true" />{{ t('Retry') }}</button></div></div>
  <p v-if="error" class="alert error" role="alert">{{ error }}</p><p v-if="actionError" class="alert error" role="alert">{{ actionError }}</p><p v-if="loading && !job" class="loading-line">{{ t('Loading job...') }}</p>
  <template v-if="job">
    <section class="detail-grid"><article class="section-block"><div class="section-heading"><div><p class="eyebrow">{{ t('Request') }}</p><h2>{{ t('Parameters') }}</h2></div></div><dl class="facts"><div><dt>{{ t('Type') }}</dt><dd>{{ valueLabel(job.kind) }}</dd></div><div><dt>{{ t('Origin') }}</dt><dd>{{ valueLabel(job.origin) }}</dd></div><div><dt>{{ t('Attempts') }}</dt><dd>{{ job.attempts }}</dd></div><div><dt>{{ t('Created') }}</dt><dd>{{ formatDateTime(job.created_at) }}</dd></div><div v-if="job.started_at"><dt>{{ t('Started') }}</dt><dd>{{ formatDateTime(job.started_at) }}</dd></div><div v-if="job.finished_at"><dt>{{ t('Finished') }}</dt><dd>{{ formatDateTime(job.finished_at) }}</dd></div></dl><pre class="json-block">{{ JSON.stringify(job.request, null, 2) }}</pre></article><article class="section-block"><div class="section-heading"><div><p class="eyebrow">{{ t('Outcome') }}</p><h2>{{ t('Result') }}</h2></div></div><p v-if="job.progress_message" class="progress-message">{{ job.progress_message }}</p><p v-if="job.state === 'running'" class="notice">{{ t('Running jobs cannot be canceled because file operations must finish cleanly.') }}</p><p v-if="job.error" class="alert error">{{ job.error }}</p><pre v-if="job.result" class="json-block">{{ JSON.stringify(job.result, null, 2) }}</pre><div v-if="artifactsFor(job).length" class="artifact-list"><span class="label">{{ t('Artifacts') }}</span><a v-for="artifact in artifactsFor(job)" :key="artifact.id" class="download-link" :href="artifact.download_url" :download="artifact.name">{{ artifact.name }} ({{ t('{size} bytes', { size: artifact.size }) }})</a></div><p v-if="!job.result && !job.error" class="empty-state">{{ t('No final result yet.') }}</p></article></section>
  </template>
</template>
