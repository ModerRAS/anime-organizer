<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { ArrowLeft, CircleSlash, RotateCcw } from 'lucide-vue-next'
import { RouterLink, useRoute } from 'vue-router'
import { api, errorMessage, type Job, type JobArtifact } from '../api'

const route = useRoute()
const job = ref<Job | null>(null)
const loading = ref(true)
const error = ref('')
const actionError = ref('')
let timer: number | undefined

async function load() {
  loading.value = true
  try { job.value = await api.job(Number(route.params.id)); error.value = '' }
  catch (reason) { error.value = errorMessage(reason) }
  finally { loading.value = false }
}
async function cancel() {
  if (!job.value) return
  actionError.value = ''
  try { job.value = await api.cancel(job.value.id) } catch (reason) { actionError.value = errorMessage(reason) }
}
async function retry() {
  if (!job.value) return
  actionError.value = ''
  try { job.value = await api.retry(job.value.id) } catch (reason) { actionError.value = errorMessage(reason) }
}
function stateLabel(value: string) { return value.replace('_', ' ') }
function dateLabel(value: string | null) { return value ? new Date(value).toLocaleString() : '-' }
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
watch(() => route.params.id, () => { void load() })
onMounted(() => { void load(); startPolling() })
onBeforeUnmount(() => { if (timer !== undefined) window.clearInterval(timer) })
</script>

<template>
  <div class="page-header detail-header"><div><RouterLink class="back-link" to="/jobs"><ArrowLeft :size="15" aria-hidden="true" />Back to jobs</RouterLink><p class="eyebrow">Job detail</p><h1 v-if="job">#{{ job.id }} <span class="state" :class="job.state">{{ stateLabel(job.state) }}</span></h1><h1 v-else>Job</h1></div><div v-if="job" class="detail-actions"><button v-if="job.state === 'queued'" class="button secondary" type="button" title="Only queued jobs can be canceled" @click="cancel"><CircleSlash :size="16" aria-hidden="true" />Cancel</button><button v-if="job.state === 'failed' || job.state === 'canceled'" class="button secondary" type="button" @click="retry"><RotateCcw :size="16" aria-hidden="true" />Retry</button></div></div>
  <p v-if="error" class="alert error" role="alert">{{ error }}</p><p v-if="actionError" class="alert error" role="alert">{{ actionError }}</p><p v-if="loading && !job" class="loading-line">Loading job...</p>
  <template v-if="job">
    <section class="detail-grid"><article class="section-block"><div class="section-heading"><div><p class="eyebrow">Request</p><h2>Parameters</h2></div></div><dl class="facts"><div><dt>Type</dt><dd>{{ job.kind }}</dd></div><div><dt>Origin</dt><dd>{{ job.origin }}</dd></div><div><dt>Attempts</dt><dd>{{ job.attempts }}</dd></div><div><dt>Created</dt><dd>{{ dateLabel(job.created_at) }}</dd></div><div v-if="job.started_at"><dt>Started</dt><dd>{{ dateLabel(job.started_at) }}</dd></div><div v-if="job.finished_at"><dt>Finished</dt><dd>{{ dateLabel(job.finished_at) }}</dd></div></dl><pre class="json-block">{{ JSON.stringify(job.request, null, 2) }}</pre></article><article class="section-block"><div class="section-heading"><div><p class="eyebrow">Outcome</p><h2>Result</h2></div></div><p v-if="job.progress_message" class="progress-message">{{ job.progress_message }}</p><p v-if="job.state === 'running'" class="notice">Running jobs cannot be canceled because file operations must finish cleanly.</p><p v-if="job.error" class="alert error">{{ job.error }}</p><pre v-if="job.result" class="json-block">{{ JSON.stringify(job.result, null, 2) }}</pre><div v-if="artifactsFor(job).length" class="artifact-list"><span class="label">Artifacts</span><a v-for="artifact in artifactsFor(job)" :key="artifact.id" class="download-link" :href="artifact.download_url" :download="artifact.name">{{ artifact.name }} ({{ artifact.size }} bytes)</a></div><p v-if="!job.result && !job.error" class="empty-state">No final result yet.</p></article></section>
  </template>
</template>
