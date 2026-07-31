<script setup lang="ts">
import { onMounted, reactive, ref, watch } from 'vue'
import { Archive, FileCheck2, FilePlus2, LoaderCircle, Play, RefreshCw, Rows3 } from 'lucide-vue-next'
import { RouterLink, useRouter } from 'vue-router'
import { api, errorMessage, type Job } from '../api'
import { formatDateTime, t, valueLabel } from '../i18n'

type Mode = 'plan' | 'apply'
type FormState = { mode: Mode; path: string; confirmed: boolean }

const router = useRouter()
const target = ref('')
const layout = reactive<FormState>({ mode: 'plan', path: '', confirmed: false })
const artwork = reactive<FormState>({ mode: 'plan', path: '', confirmed: false })
const jobs = ref<Job[]>([])
const loading = ref(false)
const submitting = ref<'layout' | 'artwork' | ''>('')
const error = ref('')

function resetConfirmation(form: FormState) {
  form.confirmed = false
}
watch(target, () => { resetConfirmation(layout); resetConfirmation(artwork) })
watch(() => [layout.mode, layout.path], () => resetConfirmation(layout))
watch(() => [artwork.mode, artwork.path], () => resetConfirmation(artwork))

function valid(form: FormState) {
  return Boolean(target.value.trim() && form.path.trim() && (form.mode === 'plan' || form.confirmed))
}

async function loadJobs() {
  loading.value = true
  try {
    const [layoutJobs, artworkJobs] = await Promise.all([
      api.jobs({ kind: 'normalize_layout', limit: 20 }),
      api.jobs({ kind: 'compact_artwork_packs', limit: 20 }),
    ])
    jobs.value = [...layoutJobs.jobs, ...artworkJobs.jobs].sort((a, b) => b.id - a.id)
    error.value = ''
  } catch (reason) {
    error.value = errorMessage(reason)
  } finally {
    loading.value = false
  }
}

async function submitLayout() {
  if (!valid(layout)) return
  submitting.value = 'layout'
  error.value = ''
  try {
    const applying = layout.mode === 'apply'
    const result = await api.enqueueNormalizeLayout({
      target: target.value.trim(),
      dry_run: !applying,
      plan: applying ? null : layout.path.trim(),
      apply_plan: applying ? layout.path.trim() : null,
    }, applying && layout.confirmed)
    await router.push(`/jobs/${result.job.id}`)
  } catch (reason) {
    error.value = errorMessage(reason)
  } finally {
    submitting.value = ''
  }
}

async function submitArtwork() {
  if (!valid(artwork)) return
  submitting.value = 'artwork'
  error.value = ''
  try {
    const applying = artwork.mode === 'apply'
    const result = await api.enqueueCompactArtworkPacks({
      target: target.value.trim(),
      dry_run: !applying,
      plan: applying ? null : artwork.path.trim(),
      apply_plan: applying ? artwork.path.trim() : null,
    }, applying && artwork.confirmed)
    await router.push(`/jobs/${result.job.id}`)
  } catch (reason) {
    error.value = errorMessage(reason)
  } finally {
    submitting.value = ''
  }
}

onMounted(loadJobs)
</script>

<template>
  <div class="page-header">
    <div><p class="eyebrow">{{ t('Library maintenance') }}</p><h1>{{ t('Maintenance') }}</h1><p class="page-subtitle">{{ t('Generate immutable plans and apply only reviewed plans through the daemon queue.') }}</p></div>
    <button class="icon-button" type="button" :title="t('Refresh maintenance jobs')" :aria-label="t('Refresh maintenance jobs')" :disabled="loading" @click="loadJobs"><RefreshCw :size="16" :class="{ spinning: loading }" aria-hidden="true" /></button>
  </div>

  <p v-if="error" class="alert error" role="alert">{{ error }}</p>

  <section class="section-block" aria-labelledby="maintenance-target-heading">
    <div class="section-heading"><div><p class="eyebrow">{{ t('Required') }}</p><h2 id="maintenance-target-heading">{{ t('Library root') }}</h2></div></div>
    <label class="form-field"><span>{{ t('Target') }} <b aria-hidden="true">*</b></span><input v-model="target" type="text" required autocomplete="off" placeholder="S:\\Anime" /></label>
  </section>

  <div class="maintenance-grid">
    <section class="section-block" aria-labelledby="layout-maintenance-heading">
      <div class="section-heading"><div><p class="eyebrow">{{ t('Layout normalization') }}</p><h2 id="layout-maintenance-heading">{{ t('Layout normalization') }}</h2></div><Rows3 :size="19" aria-hidden="true" /></div>
      <form class="maintenance-form" @submit.prevent="submitLayout">
        <div class="mode-switch" role="group" :aria-label="t('Mode')">
          <button type="button" :class="{ active: layout.mode === 'plan' }" :aria-pressed="layout.mode === 'plan'" @click="layout.mode = 'plan'"><FilePlus2 :size="16" aria-hidden="true" />{{ t('Generate plan') }}</button>
          <button type="button" :class="{ active: layout.mode === 'apply' }" :aria-pressed="layout.mode === 'apply'" @click="layout.mode = 'apply'"><FileCheck2 :size="16" aria-hidden="true" />{{ t('Apply plan') }}</button>
        </div>
        <label class="form-field"><span>{{ t(layout.mode === 'plan' ? 'Plan output path' : 'Reviewed plan path') }} <b aria-hidden="true">*</b></span><input v-model="layout.path" type="text" required autocomplete="off" placeholder="C:\\Plans\\layout.json" /></label>
        <div v-if="layout.mode === 'apply'" class="notice danger-confirmation" role="alert">
          <strong>{{ t('Confirmation required') }}</strong>
          <label class="checkbox-field"><input v-model="layout.confirmed" type="checkbox" /><span>{{ t('I confirm this reviewed plan may move or delete files and update library.db.') }}</span></label>
        </div>
        <div class="form-actions"><button class="button" :class="layout.mode === 'plan' ? 'primary' : 'danger'" type="submit" :disabled="submitting !== '' || !valid(layout)"><LoaderCircle v-if="submitting === 'layout'" class="spinning" :size="16" aria-hidden="true" /><Play v-else :size="16" aria-hidden="true" />{{ t(layout.mode === 'plan' ? 'Queue plan' : 'Queue reviewed plan') }}</button></div>
      </form>
    </section>

    <section class="section-block" aria-labelledby="artwork-maintenance-heading">
      <div class="section-heading"><div><p class="eyebrow">{{ t('Artwork pack compaction') }}</p><h2 id="artwork-maintenance-heading">{{ t('Artwork pack compaction') }}</h2></div><Archive :size="19" aria-hidden="true" /></div>
      <form class="maintenance-form" @submit.prevent="submitArtwork">
        <div class="mode-switch" role="group" :aria-label="t('Mode')">
          <button type="button" :class="{ active: artwork.mode === 'plan' }" :aria-pressed="artwork.mode === 'plan'" @click="artwork.mode = 'plan'"><FilePlus2 :size="16" aria-hidden="true" />{{ t('Generate plan') }}</button>
          <button type="button" :class="{ active: artwork.mode === 'apply' }" :aria-pressed="artwork.mode === 'apply'" @click="artwork.mode = 'apply'"><FileCheck2 :size="16" aria-hidden="true" />{{ t('Apply plan') }}</button>
        </div>
        <label class="form-field"><span>{{ t(artwork.mode === 'plan' ? 'Plan output path' : 'Reviewed plan path') }} <b aria-hidden="true">*</b></span><input v-model="artwork.path" type="text" required autocomplete="off" placeholder="C:\\Plans\\artwork.json" /></label>
        <div v-if="artwork.mode === 'apply'" class="notice danger-confirmation" role="alert">
          <strong>{{ t('Confirmation required') }}</strong>
          <label class="checkbox-field"><input v-model="artwork.confirmed" type="checkbox" /><span>{{ t('I confirm this reviewed plan may replace artwork packs and update library.db.') }}</span></label>
        </div>
        <div class="form-actions"><button class="button" :class="artwork.mode === 'plan' ? 'primary' : 'danger'" type="submit" :disabled="submitting !== '' || !valid(artwork)"><LoaderCircle v-if="submitting === 'artwork'" class="spinning" :size="16" aria-hidden="true" /><Play v-else :size="16" aria-hidden="true" />{{ t(artwork.mode === 'plan' ? 'Queue plan' : 'Queue reviewed plan') }}</button></div>
      </form>
    </section>
  </div>

  <section class="section-block" aria-labelledby="maintenance-history-heading">
    <div class="section-heading"><div><p class="eyebrow">{{ t('History') }}</p><h2 id="maintenance-history-heading">{{ t('Recent maintenance jobs') }}</h2></div><span class="record-count">{{ t('{count} records', { count: jobs.length }) }}</span></div>
    <div class="table-wrap"><table><thead><tr><th>{{ t('Job') }}</th><th>{{ t('Type') }}</th><th>{{ t('State') }}</th><th>{{ t('Created') }}</th><th>{{ t('Finished') }}</th></tr></thead><tbody>
      <tr v-for="job in jobs" :key="job.id"><td><RouterLink :to="`/jobs/${job.id}`">#{{ job.id }}</RouterLink></td><td>{{ valueLabel(job.kind) }}</td><td><span class="state" :class="job.state">{{ valueLabel(job.state) }}</span></td><td>{{ formatDateTime(job.created_at) }}</td><td>{{ formatDateTime(job.finished_at) }}</td></tr>
      <tr v-if="!loading && !jobs.length"><td colspan="5" class="empty-cell">{{ t('No maintenance jobs yet.') }}</td></tr>
    </tbody></table></div>
    <p v-if="loading" class="loading-line">{{ t('Refreshing maintenance jobs...') }}</p>
  </section>
</template>
