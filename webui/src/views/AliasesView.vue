<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { Download, Play, RefreshCw, Tags } from 'lucide-vue-next'
import { RouterLink, useRouter } from 'vue-router'
import { api, errorMessage, type Job, type JobArtifact } from '../api'
import { useStatus } from '../stores/status'

const router = useRouter()
const { state } = useStatus()
const buildOutput = ref('bangumi.db')
const includeRelations = ref(false)
const buildVerbose = ref(false)
const extractInput = ref('')
const extractDownload = ref(false)
const extractOutput = ref('')
const mergeInput = ref('')
const mergeTarget = ref('bangumi.db')
const applyInput = ref('')
const applyTarget = ref('bangumi.db')
const issueInput = ref('')
const issueRepo = ref('')
const confirmed = ref(false)
const jobs = ref<Job[]>([])
const loading = ref(false)
const submitting = ref(false)
const error = ref('')

const issueAvailable = computed(() => state.capabilities?.job_types.includes('create_alias_issues') === true)
const kinds = ['build_bangumi_db', 'extract_aliases', 'merge_aliases', 'apply_matches', 'create_alias_issues']

function artifacts(job: Job): JobArtifact[] {
  const result = job.result
  return result && typeof result === 'object' && Array.isArray((result as { artifacts?: unknown }).artifacts)
    ? (result as { artifacts: JobArtifact[] }).artifacts : []
}
async function load() {
  loading.value = true
  try {
    const loaded = await Promise.all(kinds.map(kind => api.jobs({ kind, limit: 20 })))
    jobs.value = loaded.flatMap(value => value.jobs).sort((a, b) => b.id - a.id)
    error.value = ''
  } catch (reason) { error.value = errorMessage(reason) }
  finally { loading.value = false }
}
async function submit(action: () => Promise<{ job: Job }>) {
  submitting.value = true
  try { await router.push(`/jobs/${(await action()).job.id}`) }
  catch (reason) { error.value = errorMessage(reason) }
  finally { submitting.value = false }
}
function requireConfirmation() { return confirmed.value }
function stateLabel(value: string) { return value.replaceAll('_', ' ') }
function dateLabel(value: string) { return new Date(value).toLocaleString() }
onMounted(load)
</script>

<template>
  <div class="page-header">
    <div><p class="eyebrow">Alias maintenance</p><h1>Aliases</h1><p class="page-subtitle">Build, extract, review, and maintain the Bangumi alias database through typed queued jobs.</p></div>
    <button class="icon-button" type="button" title="Refresh alias jobs" aria-label="Refresh alias jobs" :disabled="loading" @click="load"><RefreshCw :size="16" :class="{ spinning: loading }" aria-hidden="true" /></button>
  </div>
  <p v-if="error" class="alert error" role="alert">{{ error }}</p>

  <section class="scraper-forms aliases-forms" aria-label="Alias maintenance actions">
    <form class="section-block scraper-form" @submit.prevent="submit(() => api.enqueueBuildBangumiDb({ output: buildOutput.trim(), include_relations: includeRelations, verbose: buildVerbose }, requireConfirmation()))">
      <div class="section-heading"><div><p class="eyebrow">Database</p><h2>Build Bangumi database</h2></div><Tags :size="19" aria-hidden="true" /></div>
      <label class="form-field"><span>Output path</span><input v-model="buildOutput" type="text" required /></label>
      <label class="checkbox-field"><input v-model="includeRelations" type="checkbox" />Include relations</label>
      <label class="checkbox-field"><input v-model="buildVerbose" type="checkbox" />Verbose worker output</label>
      <div class="form-actions"><span class="form-hint">Downloads the current Bangumi archive.</span><button class="button primary" type="submit" :disabled="submitting || !buildOutput.trim()"><Play :size="16" aria-hidden="true" />Queue build</button></div>
    </form>

    <form class="section-block scraper-form" @submit.prevent="submit(() => api.enqueueExtractAliases({ input: extractInput.trim() || null, download: extractDownload, output: extractOutput.trim() || null }))">
      <div class="section-heading"><div><p class="eyebrow">Extraction</p><h2>Extract aliases</h2></div><Tags :size="19" aria-hidden="true" /></div>
      <label class="form-field"><span>Dump path <small>(optional when downloading)</small></span><input v-model="extractInput" type="text" /></label>
      <label class="form-field"><span>Output path <small>(optional)</small></span><input v-model="extractOutput" type="text" /></label>
      <label class="checkbox-field"><input v-model="extractDownload" type="checkbox" />Download latest dump</label>
      <div class="form-actions"><span class="form-hint">The JSON result is saved as a job artifact.</span><button class="button primary" type="submit" :disabled="submitting || (!extractDownload && !extractInput.trim())"><Play :size="16" aria-hidden="true" />Queue extraction</button></div>
    </form>

    <form class="section-block scraper-form" @submit.prevent="submit(() => api.enqueueMergeAliases({ input: mergeInput.trim(), target: mergeTarget.trim() || null }, requireConfirmation()))">
      <div class="section-heading"><div><p class="eyebrow">Mutation</p><h2>Merge aliases</h2></div><Tags :size="19" aria-hidden="true" /></div>
      <label class="form-field"><span>Alias JSON path</span><input v-model="mergeInput" type="text" required /></label>
      <label class="form-field"><span>Target database</span><input v-model="mergeTarget" type="text" required /></label>
      <div class="form-actions"><span class="form-hint">Writes only to the selected target database.</span><button class="button primary" type="submit" :disabled="submitting || !mergeInput.trim() || !requireConfirmation()"><Play :size="16" aria-hidden="true" />Queue merge</button></div>
    </form>

    <form class="section-block scraper-form" @submit.prevent="submit(() => api.enqueueApplyMatches({ input: applyInput.trim(), target: applyTarget.trim() || null }, requireConfirmation()))">
      <div class="section-heading"><div><p class="eyebrow">Mutation</p><h2>Apply matches</h2></div><Tags :size="19" aria-hidden="true" /></div>
      <label class="form-field"><span>Proposal JSON path</span><input v-model="applyInput" type="text" required /></label>
      <label class="form-field"><span>Target database</span><input v-model="applyTarget" type="text" required /></label>
      <div class="form-actions"><span class="form-hint">Only confident proposals are applied.</span><button class="button primary" type="submit" :disabled="submitting || !applyInput.trim() || !requireConfirmation()"><Play :size="16" aria-hidden="true" />Queue apply</button></div>
    </form>

    <form v-if="issueAvailable" class="section-block scraper-form" @submit.prevent="submit(() => api.enqueueCreateAliasIssues({ input: issueInput.trim(), repo: issueRepo.trim() || null }, requireConfirmation()))">
      <div class="section-heading"><div><p class="eyebrow">Review</p><h2>Create GitHub issues</h2></div><Tags :size="19" aria-hidden="true" /></div>
      <label class="form-field"><span>Uncertain proposal JSON path</span><input v-model="issueInput" type="text" required /></label>
      <label class="form-field"><span>Repository <small>(optional)</small></span><input v-model="issueRepo" type="text" placeholder="owner/name" /></label>
      <div class="form-actions"><span class="form-hint">Each proposal receives an individual result.</span><button class="button primary" type="submit" :disabled="submitting || !issueInput.trim() || !requireConfirmation()"><Play :size="16" aria-hidden="true" />Queue issue creation</button></div>
    </form>
  </section>

  <label class="checkbox-field confirmation-field"><input v-model="confirmed" type="checkbox" />I confirm database mutations and external issue creation</label>

  <section class="section-block" aria-labelledby="alias-history-heading">
    <div class="section-heading"><div><p class="eyebrow">History</p><h2 id="alias-history-heading">Maintenance jobs</h2></div><span class="record-count">{{ jobs.length }} records</span></div>
    <div class="table-wrap"><table><thead><tr><th>Job</th><th>Type</th><th>State</th><th>Created</th><th>Downloads</th></tr></thead><tbody>
      <tr v-for="job in jobs" :key="job.id"><td><RouterLink :to="`/jobs/${job.id}`">#{{ job.id }}</RouterLink></td><td>{{ stateLabel(job.kind) }}</td><td><span class="state" :class="job.state">{{ stateLabel(job.state) }}</span></td><td>{{ dateLabel(job.created_at) }}</td><td><span v-if="!artifacts(job).length" class="table-subtext">Inline result</span><a v-for="artifact in artifacts(job)" :key="artifact.id" class="download-link" :href="artifact.download_url" :download="artifact.name"><Download :size="14" aria-hidden="true" />{{ artifact.name }}</a></td></tr>
      <tr v-if="!loading && !jobs.length"><td colspan="5" class="empty-cell">No alias maintenance jobs yet.</td></tr>
    </tbody></table></div>
    <p v-if="loading" class="loading-line">Refreshing alias history...</p>
  </section>
</template>
