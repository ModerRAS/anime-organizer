<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { Download, FileSearch, Play, RefreshCw } from 'lucide-vue-next'
import { RouterLink, useRouter } from 'vue-router'
import { api, errorMessage, type Job, type JobArtifact } from '../api'

const router = useRouter()
const days = ref(7)
const format = ref<'json' | 'pretty'>('json')
const tmdbApiKey = ref('')
const matchInput = ref('')
const matchFormat = ref<'json' | 'github'>('github')
const jobs = ref<Job[]>([])
const loading = ref(false)
const submitting = ref(false)
const error = ref('')

function artifacts(job: Job): JobArtifact[] {
  const result = job.result
  if (!result || typeof result !== 'object' || !Array.isArray((result as { artifacts?: unknown }).artifacts)) return []
  return (result as { artifacts: JobArtifact[] }).artifacts
}

async function load() {
  loading.value = true
  try {
    const [scrapes, matches] = await Promise.all([
      api.jobs({ kind: 'scrape', limit: 20 }),
      api.jobs({ kind: 'match_aliases', limit: 20 }),
    ])
    jobs.value = [...scrapes.jobs, ...matches.jobs].sort((a, b) => b.id - a.id)
    error.value = ''
  } catch (reason) { error.value = errorMessage(reason) }
  finally { loading.value = false }
}

async function submitScrape() {
  submitting.value = true
  try {
    const result = await api.enqueueScrape({ days: Math.max(0, Math.min(3650, days.value)), format: format.value, tmdb_api_key: tmdbApiKey.value || null })
    await router.push(`/jobs/${result.job.id}`)
  } catch (reason) { error.value = errorMessage(reason) }
  finally { submitting.value = false }
}

async function submitMatch() {
  submitting.value = true
  try {
    const result = await api.enqueueMatchAliases({ input: matchInput.value.trim(), format: matchFormat.value })
    await router.push(`/jobs/${result.job.id}`)
  } catch (reason) { error.value = errorMessage(reason) }
  finally { submitting.value = false }
}

function stateLabel(value: string) { return value.replace('_', ' ') }
function dateLabel(value: string) { return new Date(value).toLocaleString() }
onMounted(load)
</script>

<template>
  <div class="page-header">
    <div><p class="eyebrow">Data workflows</p><h1>Scraper</h1><p class="page-subtitle">Run bounded scrape and alias matching jobs through the daemon queue.</p></div>
    <button class="icon-button" type="button" title="Refresh scraper jobs" aria-label="Refresh scraper jobs" :disabled="loading" @click="load"><RefreshCw :size="16" :class="{ spinning: loading }" aria-hidden="true" /></button>
  </div>
  <p v-if="error" class="alert error" role="alert">{{ error }}</p>

  <section class="scraper-forms" aria-label="Scraper actions">
    <form class="section-block scraper-form" @submit.prevent="submitScrape">
      <div class="section-heading"><div><p class="eyebrow">Sources</p><h2>Scrape recent anime</h2></div><FileSearch :size="19" aria-hidden="true" /></div>
      <div class="form-grid">
        <label class="form-field"><span>Days</span><input v-model.number="days" type="number" min="0" max="3650" required /></label>
        <label class="form-field"><span>Output mode</span><select v-model="format"><option value="json">JSON</option><option value="pretty">Pretty</option></select></label>
      </div>
      <label class="form-field"><span>TMDB API key <small>(optional)</small></span><input v-model="tmdbApiKey" type="password" autocomplete="off" /></label>
      <div class="form-actions"><span class="form-hint">Results stay in the job record or an artifact.</span><button class="button primary" type="submit" :disabled="submitting"><Play :size="16" aria-hidden="true" />Queue scrape</button></div>
    </form>

    <form class="section-block scraper-form" @submit.prevent="submitMatch">
      <div class="section-heading"><div><p class="eyebrow">Matching</p><h2>Match aliases</h2></div><FileSearch :size="19" aria-hidden="true" /></div>
      <label class="form-field"><span>Scrape JSON path</span><input v-model="matchInput" type="text" placeholder="C:\\data\\scraped.json" required /></label>
      <label class="form-field"><span>Result mode</span><select v-model="matchFormat"><option value="github">JSON + GitHub artifact</option><option value="json">JSON</option></select></label>
      <div class="form-actions"><span class="form-hint">The worker reads the local input when the job starts.</span><button class="button primary" type="submit" :disabled="submitting || !matchInput.trim()"><Play :size="16" aria-hidden="true" />Queue matching</button></div>
    </form>
  </section>

  <section class="section-block" aria-labelledby="scraper-history-heading">
    <div class="section-heading"><div><p class="eyebrow">History</p><h2 id="scraper-history-heading">Scraper jobs</h2></div><span class="record-count">{{ jobs.length }} records</span></div>
    <div class="table-wrap"><table><thead><tr><th>Job</th><th>Type</th><th>State</th><th>Created</th><th>Downloads</th></tr></thead><tbody>
      <tr v-for="job in jobs" :key="job.id"><td><RouterLink :to="`/jobs/${job.id}`">#{{ job.id }}</RouterLink></td><td>{{ stateLabel(job.kind) }}</td><td><span class="state" :class="job.state">{{ stateLabel(job.state) }}</span></td><td>{{ dateLabel(job.created_at) }}</td><td><span v-if="!artifacts(job).length" class="table-subtext">Inline result</span><a v-for="artifact in artifacts(job)" :key="artifact.id" class="download-link" :href="artifact.download_url" :download="artifact.name"><Download :size="14" aria-hidden="true" />{{ artifact.name }}</a></td></tr>
      <tr v-if="!loading && !jobs.length"><td colspan="5" class="empty-cell">No scraper jobs yet.</td></tr>
    </tbody></table></div>
    <p v-if="loading" class="loading-line">Refreshing scraper history...</p>
  </section>
</template>
