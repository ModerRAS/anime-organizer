<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { Download, FileSearch, Play, RefreshCw } from 'lucide-vue-next'
import { RouterLink, useRouter } from 'vue-router'
import { api, errorMessage, type Job, type JobArtifact } from '../api'

const router = useRouter()
const source = ref<'dmhy' | 'nyaa' | 'all'>('all')
const query = ref('')
const pages = ref(1)
const output = ref('')
const jobs = ref<Job[]>([])
const loading = ref(false)
const submitting = ref(false)
const error = ref('')

function artifacts(job: Job): JobArtifact[] {
  const result = job.result
  return result && typeof result === 'object' && Array.isArray((result as { artifacts?: unknown }).artifacts)
    ? (result as { artifacts: JobArtifact[] }).artifacts : []
}
function resultData(job: Job): { count?: number; preview?: string[]; preview_truncated?: boolean } | null {
  const result = job.result
  if (!result || typeof result !== 'object' || !('data' in result) || !result.data || typeof result.data !== 'object') return null
  return result.data as { count?: number; preview?: string[]; preview_truncated?: boolean }
}
function clampedPages(value: number) { return Math.max(1, Math.min(2000, Math.trunc(Number.isFinite(value) ? value : 1))) }
function jobSource(job: Job) {
  const request = job.request
  return request && typeof request === 'object' && 'args' in request && request.args && typeof request.args === 'object' && 'source' in request.args
    ? String(request.args.source) : '-'
}
async function load() {
  loading.value = true
  try {
    jobs.value = (await api.jobs({ kind: 'torrent_scrape', limit: 30 })).jobs
    error.value = ''
  } catch (reason) { error.value = errorMessage(reason) }
  finally { loading.value = false }
}
async function submit() {
  submitting.value = true
  try {
    const result = await api.enqueueTorrentScrape({
      source: source.value,
      query: source.value === 'nyaa' ? query.value.trim() || null : null,
      pages: clampedPages(pages.value),
      output: output.value.trim() || null,
      headed: false,
    })
    await router.push(`/jobs/${result.job.id}`)
  } catch (reason) { error.value = errorMessage(reason) }
  finally { submitting.value = false }
}
function stateLabel(value: string) { return value.replaceAll('_', ' ') }
function dateLabel(value: string) { return new Date(value).toLocaleString() }
onMounted(load)
</script>

<template>
  <div class="page-header">
    <div><p class="eyebrow">Torrent sources</p><h1>DMHY and Nyaa</h1><p class="page-subtitle">Queue bounded source scrapes and review unique torrent filenames.</p></div>
    <button class="icon-button" type="button" title="Refresh torrent jobs" aria-label="Refresh torrent jobs" :disabled="loading" @click="load"><RefreshCw :size="16" :class="{ spinning: loading }" aria-hidden="true" /></button>
  </div>
  <p v-if="error" class="alert error" role="alert">{{ error }}</p>

  <section class="section-block scraper-form" aria-labelledby="torrent-form-heading">
    <div class="section-heading"><div><p class="eyebrow">New scrape</p><h2 id="torrent-form-heading">Source parameters</h2></div><FileSearch :size="19" aria-hidden="true" /></div>
    <form class="organize-form" @submit.prevent="submit">
      <div class="form-grid">
        <label class="form-field"><span>Source</span><select v-model="source"><option value="all">DMHY and Nyaa</option><option value="dmhy">DMHY</option><option value="nyaa">Nyaa</option></select></label>
        <label class="form-field"><span>Pages</span><input v-model.number="pages" type="number" min="1" max="2000" required /></label>
      </div>
      <label v-if="source === 'nyaa'" class="form-field"><span>Nyaa search query <small>(optional)</small></span><input v-model="query" type="search" placeholder="anime" /></label>
      <label class="form-field"><span>Output path <small>(optional)</small></span><input v-model="output" type="text" placeholder="C:\\data\\torrent-titles.txt" /></label>
      <div class="form-actions"><span class="form-hint">Pages are clamped to 1-2000. Results include a text artifact and bounded preview.</span><button class="button primary" type="submit" :disabled="submitting"><Play :size="16" aria-hidden="true" />Queue scrape</button></div>
    </form>
  </section>

  <section class="section-block" aria-labelledby="torrent-history-heading">
    <div class="section-heading"><div><p class="eyebrow">History</p><h2 id="torrent-history-heading">Torrent scrape jobs</h2></div><span class="record-count">{{ jobs.length }} records</span></div>
    <div class="table-wrap"><table><thead><tr><th>Job</th><th>Source</th><th>State</th><th>Unique titles</th><th>Preview</th><th>Artifact</th></tr></thead><tbody>
      <tr v-for="job in jobs" :key="job.id">
        <td><RouterLink :to="`/jobs/${job.id}`">#{{ job.id }}</RouterLink><small class="table-subtext">{{ dateLabel(job.created_at) }}</small></td>
        <td>{{ jobSource(job) }}</td>
        <td><span class="state" :class="job.state">{{ stateLabel(job.state) }}</span></td>
        <td>{{ resultData(job)?.count ?? '-' }}</td>
        <td><ol v-if="resultData(job)?.preview?.length" class="torrent-preview"><li v-for="line in resultData(job)?.preview" :key="line">{{ line }}</li></ol><span v-if="resultData(job)?.preview_truncated" class="table-subtext">Preview truncated</span><span v-if="!resultData(job)?.preview?.length" class="table-subtext">No result yet</span></td>
        <td><span v-if="!artifacts(job).length" class="table-subtext">-</span><a v-for="artifact in artifacts(job)" :key="artifact.id" class="download-link" :href="artifact.download_url" :download="artifact.name"><Download :size="14" aria-hidden="true" />{{ artifact.name }}</a></td>
      </tr>
      <tr v-if="!loading && !jobs.length"><td colspan="6" class="empty-cell">No torrent scrape jobs yet.</td></tr>
    </tbody></table></div>
    <p v-if="loading" class="loading-line">Refreshing torrent history...</p>
  </section>
</template>
