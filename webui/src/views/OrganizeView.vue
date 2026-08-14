<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { ChevronRight, LoaderCircle, Save, Send, Trash2 } from 'lucide-vue-next'
import { useRouter } from 'vue-router'
import { api, errorMessage, type Connection, type StorageEndpoint } from '../api'
import { loadCapabilities } from '../stores/status'
import { t } from '../i18n'
import {
  defaultOrganizeForm,
  deleteOrganizePreset,
  fieldErrors,
  loadOrganizePresets,
  saveOrganizePreset,
  toOrganizeArgs,
  validateOrganize,
  type OrganizeForm,
  type OrganizePreset,
} from '../organize'

const router = useRouter()
const view = ref<'local' | 'storage'>('local')
const storageAvailable = ref(false)
const connections = ref<Connection[]>([])
const storageForm = reactive({
  sourceType: 'local' as 'local' | 'connection',
  sourceConnectionId: null as number | null,
  sourcePath: '',
  targetType: 'connection' as 'local' | 'connection',
  targetConnectionId: null as number | null,
  targetPath: '/',
  mode: 'move' as 'move' | 'copy',
  seasonMode: true,
  mlip: false,
  removeEmptyDirs: true,
})
const storageConfirmed = ref(false)
const storageSubmitting = ref(false)
const form = reactive<OrganizeForm>(defaultOrganizeForm())
const errors = ref<Record<string, string>>({})
const apiError = ref('')
const submitting = ref(false)
const confirmed = ref(false)
const presets = ref<OrganizePreset[]>(loadOrganizePresets())
const selectedPreset = ref('')
const presetName = ref('')
const presetError = ref('')
const needsConfirmation = computed(() => form.mode === 'move' || form.rebuild_library_index)

watch(storageAvailable, (available) => {
  if (!available) view.value = 'local'
})

watch(needsConfirmation, (required, wasRequired) => {
  if (required !== wasRequired) confirmed.value = false
})

function errorFor(field: string) {
  const message = errors.value[field]
  return message ? t(message) : ''
}

function loadPreset() {
  const preset = presets.value.find((item) => item.name === selectedPreset.value)
  if (!preset) return
  Object.assign(form, preset.form)
  errors.value = {}
  apiError.value = ''
  confirmed.value = false
}

function savePreset() {
  const name = presetName.value.trim()
  if (!name) {
    presetError.value = t('Preset name is required.')
    return
  }
  presets.value = saveOrganizePreset(name, form)
  selectedPreset.value = name
  presetName.value = ''
  presetError.value = ''
}

function deletePreset() {
  if (!selectedPreset.value || !window.confirm(t('Delete this preset?'))) return
  presets.value = deleteOrganizePreset(selectedPreset.value)
  selectedPreset.value = ''
}

async function submitStorage() {
  apiError.value = ''
  const endpoint = (type: 'local' | 'connection', connectionId: number | null, path: string): StorageEndpoint | null => {
    const normalized = path.trim()
    if (!normalized || (type === 'connection' && connectionId === null)) return null
    return type === 'local'
      ? { type: 'local', path: normalized }
      : { type: 'connection', connection_id: connectionId as number, path: normalized }
  }
  const source = endpoint(storageForm.sourceType, storageForm.sourceConnectionId, storageForm.sourcePath)
  const target = endpoint(storageForm.targetType, storageForm.targetConnectionId, storageForm.targetPath)
  if (!source || !target) {
    apiError.value = t('Both storage endpoints and their paths are required.')
    return
  }
  if ((storageForm.mode === 'move' || storageForm.removeEmptyDirs) && !storageConfirmed.value) {
    apiError.value = t('Confirm this storage move before submitting.')
    return
  }
  storageSubmitting.value = true
  try {
    const result = await api.enqueueStorageOrganize({
      source,
      target,
      mode: storageForm.mode,
      season_mode: storageForm.seasonMode,
      mlip: storageForm.mlip,
      remove_empty_dirs: storageForm.removeEmptyDirs,
    }, storageConfirmed.value)
    await router.push(`/jobs/${result.job.id}`)
  } catch (reason) {
    apiError.value = errorMessage(reason)
  } finally {
    storageSubmitting.value = false
  }
}

onMounted(async () => {
  try {
    const capabilities = await loadCapabilities()
    storageAvailable.value = capabilities.job_types.includes('storage_organize')
    if (storageAvailable.value) connections.value = (await api.connections()).connections
  } catch (reason) {
    storageAvailable.value = false
    apiError.value = errorMessage(reason)
  }
})

async function submit() {
  apiError.value = ''
  errors.value = validateOrganize(form, confirmed.value)
  if (Object.keys(errors.value).length) return

  submitting.value = true
  try {
    const result = await api.enqueueOrganize(toOrganizeArgs(form), confirmed.value)
    await router.push(`/jobs/${result.job.id}`)
  } catch (reason) {
    apiError.value = errorMessage(reason)
    errors.value = { ...errors.value, ...fieldErrors(apiError.value) }
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <div class="page-header">
    <div><p class="eyebrow">{{ t('Manual job') }}</p><h1>{{ t('Organize') }}</h1><p class="page-subtitle">{{ t('Submit one complete organize request to the daemon queue.') }}</p></div>
  </div>

  <div v-if="storageAvailable" class="organize-tabs" role="tablist" :aria-label="t('Organization source type')">
    <button type="button" role="tab" :class="{ active: view === 'local' }" :aria-selected="view === 'local'" @click="view = 'local'">{{ t('Local filesystem') }}</button>
    <button type="button" role="tab" :class="{ active: view === 'storage' }" :aria-selected="view === 'storage'" @click="view = 'storage'">{{ t('Cross-storage') }}</button>
  </div>

  <p v-if="apiError" class="alert error" role="alert">{{ apiError }}</p>
  <section v-if="view === 'local'" class="section-block preset-section" aria-labelledby="preset-heading">
    <div class="section-heading"><div><p class="eyebrow">{{ t('Defaults') }}</p><h2 id="preset-heading">{{ t('Presets') }}</h2></div></div>
    <div class="preset-bar">
      <label class="form-field preset-select"><span>{{ t('Saved preset') }}</span><select v-model="selectedPreset" @change="loadPreset"><option value="">{{ t('Choose a preset') }}</option><option v-for="preset in presets" :key="preset.name" :value="preset.name">{{ preset.name }}</option></select></label>
      <button class="button secondary" type="button" :disabled="!selectedPreset" @click="loadPreset"><ChevronRight :size="16" aria-hidden="true" />{{ t('Load') }}</button>
      <button class="icon-button danger-action" type="button" :title="t('Delete selected preset')" :aria-label="t('Delete selected preset')" :disabled="!selectedPreset" @click="deletePreset"><Trash2 :size="16" aria-hidden="true" /></button>
      <label class="form-field preset-name"><span>{{ t('New preset name') }}</span><input v-model="presetName" type="text" maxlength="80" :placeholder="t('Optional saved name')" @keyup.enter="savePreset" /><small v-if="presetError" class="field-error" role="alert">{{ presetError }}</small></label>
      <button class="button secondary" type="button" @click="savePreset"><Save :size="16" aria-hidden="true" />{{ t('Save preset') }}</button>
    </div>
  </section>

  <form v-if="view === 'local'" class="organize-form" novalidate @submit.prevent="submit">
    <section class="section-block" aria-labelledby="paths-heading">
      <div class="section-heading"><div><p class="eyebrow">{{ t('Required') }}</p><h2 id="paths-heading">{{ t('Paths and operation') }}</h2></div></div>
      <div class="form-grid">
        <label class="form-field"><span>{{ t('Source') }} <b aria-hidden="true">*</b></span><input v-model="form.source" type="text" autocomplete="off" placeholder="C:\\Downloads\\Anime" :aria-invalid="Boolean(errorFor('source'))" aria-describedby="source-error" /><small v-if="errorFor('source')" id="source-error" class="field-error" role="alert">{{ errorFor('source') }}</small></label>
        <label class="form-field"><span>{{ t('Target') }} <b aria-hidden="true">*</b></span><input v-model="form.target" type="text" autocomplete="off" placeholder="S:\\Anime" :aria-invalid="Boolean(errorFor('target'))" aria-describedby="target-error" /><small v-if="errorFor('target')" id="target-error" class="field-error" role="alert">{{ errorFor('target') }}</small></label>
        <label class="form-field"><span>{{ t('Mode') }}</span><select v-model="form.mode" :aria-invalid="Boolean(errorFor('mode'))"><option value="link">{{ t('Hard link') }}</option><option value="copy">{{ t('Copy') }}</option><option value="move">{{ t('Move') }}</option></select><small v-if="errorFor('mode')" class="field-error" role="alert">{{ errorFor('mode') }}</small></label>
        <label class="form-field"><span>{{ t('Link failure fallback') }}</span><select v-model="form.fallback_on_link_failure"><option value="">{{ t('No fallback') }}</option><option value="move">{{ t('Move') }}</option><option value="copy">{{ t('Copy') }}</option></select><small v-if="errorFor('fallback_on_link_failure')" class="field-error" role="alert">{{ errorFor('fallback_on_link_failure') }}</small></label>
      </div>
      <label class="checkbox-field"><input v-model="form.dry_run" type="checkbox" /><span>{{ t('Dry run') }}</span></label>
    </section>

    <details class="section-block advanced-options">
      <summary><span><span class="eyebrow">{{ t('Optional') }}</span><strong>{{ t('Advanced options') }}</strong></span></summary>
      <div class="form-grid">
        <label class="form-field"><span>{{ t('Included extensions') }}</span><input v-model="form.include_ext" type="text" :placeholder="t('mp4,mkv,avi')" :aria-invalid="Boolean(errorFor('include_ext'))" /><small v-if="errorFor('include_ext')" class="field-error" role="alert">{{ errorFor('include_ext') }}</small></label>
        <label class="form-field"><span>{{ t('Filename parser') }}</span><select v-model="form.filename_parser" :aria-invalid="Boolean(errorFor('filename_parser'))"><option value="rules">{{ t('Rules') }}</option><option value="anifilebert">{{ t('AniFileBERT') }}</option><option value="auto">{{ t('Auto') }}</option></select><small v-if="errorFor('filename_parser')" class="field-error" role="alert">{{ errorFor('filename_parser') }}</small></label>
        <label class="form-field"><span>{{ t('TMDB API key') }}</span><input v-model="form.tmdb_api_key" type="password" autocomplete="off" :placeholder="t('Optional key')" :aria-invalid="Boolean(errorFor('tmdb_api_key'))" /><small v-if="errorFor('tmdb_api_key')" class="field-error" role="alert">{{ errorFor('tmdb_api_key') }}</small></label>
        <label class="form-field"><span>{{ t('Alias file') }}</span><input v-model="form.alias_file" type="text" autocomplete="off" :placeholder="t('Optional JSON path')" :aria-invalid="Boolean(errorFor('alias_file'))" /><small v-if="errorFor('alias_file')" class="field-error" role="alert">{{ errorFor('alias_file') }}</small></label>
        <label class="form-field"><span>{{ t('Bangumi cache') }}</span><input v-model="form.bangumi_cache" type="text" autocomplete="off" :placeholder="t('Optional cache path')" :aria-invalid="Boolean(errorFor('bangumi_cache'))" /><small v-if="errorFor('bangumi_cache')" class="field-error" role="alert">{{ errorFor('bangumi_cache') }}</small></label>
        <label class="form-field"><span>{{ t('Metadata source') }}</span><input v-model="form.metadata_source" type="text" autocomplete="off" :placeholder="t('Optional local subject.jsonlines path')" :aria-invalid="Boolean(errorFor('metadata_source'))" /><small v-if="errorFor('metadata_source')" class="field-error" role="alert">{{ errorFor('metadata_source') }}</small></label>
      </div>
      <div class="checkbox-grid">
        <label class="checkbox-field"><input v-model="form.verbose" type="checkbox" /><span>{{ t('Verbose logging') }}</span></label>
        <label class="checkbox-field"><input v-model="form.scrape_metadata" type="checkbox" /><span>{{ t('Scrape metadata') }}</span></label>
        <label class="checkbox-field"><input v-model="form.no_images" type="checkbox" /><span>{{ t('Skip images') }}</span></label>
        <label class="checkbox-field"><input v-model="form.no_episode_metadata" type="checkbox" /><span>{{ t('Skip episode metadata') }}</span></label>
        <label class="checkbox-field"><input v-model="form.force_overwrite" type="checkbox" /><span>{{ t('Overwrite existing metadata') }}</span></label>
        <label class="checkbox-field"><input v-model="form.season_mode" type="checkbox" /><span>{{ t('Season mode') }}</span></label>
        <label class="checkbox-field"><input v-model="form.library_index" type="checkbox" /><span>{{ t('Update library index') }}</span></label>
        <label class="checkbox-field"><input v-model="form.mlip" type="checkbox" /><span>{{ t('Build MLIP library') }}</span></label>
        <label class="checkbox-field"><input v-model="form.rebuild_library_index" type="checkbox" :aria-invalid="Boolean(errorFor('rebuild_library_index'))" /><span>{{ t('Rebuild library index') }}<small v-if="errorFor('rebuild_library_index')" class="field-error" role="alert">{{ errorFor('rebuild_library_index') }}</small></span></label>
        <label class="checkbox-field"><input v-model="form.probe_runtime" type="checkbox" /><span>{{ t('Probe runtime with ffprobe') }}</span></label>
      </div>
    </details>

    <div v-if="needsConfirmation" class="notice danger-confirmation" role="alert">
      <strong>{{ t('Confirmation required') }}</strong>
      <label class="checkbox-field"><input v-model="confirmed" type="checkbox" :aria-invalid="Boolean(errorFor('confirmed'))" /><span>{{ t('I understand this request can change files or rebuild the library index.') }}</span></label>
      <small v-if="errorFor('confirmed')" class="field-error" role="alert">{{ errorFor('confirmed') }}</small>
    </div>
    <div class="form-actions"><button class="button primary" type="submit" :disabled="submitting"><LoaderCircle v-if="submitting" class="spinning" :size="16" aria-hidden="true" /><Send v-else :size="16" aria-hidden="true" />{{ submitting ? t('Submitting...') : t('Submit organize job') }}</button><span class="form-hint">{{ t('Required fields are marked *') }}</span></div>
  </form>

  <form v-if="view === 'storage' && storageAvailable" class="organize-form" @submit.prevent="submitStorage">
    <section class="section-block" aria-labelledby="storage-paths-heading">
      <div class="section-heading"><div><p class="eyebrow">{{ t('Required') }}</p><h2 id="storage-paths-heading">{{ t('Storage endpoints') }}</h2></div></div>
      <div class="form-grid">
        <label class="form-field"><span>{{ t('Source type') }}</span><select v-model="storageForm.sourceType"><option value="local">{{ t('Local filesystem') }}</option><option value="connection">{{ t('Saved connection') }}</option></select></label>
        <label v-if="storageForm.sourceType === 'connection'" class="form-field"><span>{{ t('Source connection') }}</span><select v-model.number="storageForm.sourceConnectionId" required><option :value="null" disabled>{{ t('Choose a connection') }}</option><option v-for="connection in connections" :key="connection.id" :value="connection.id">{{ connection.name }} ({{ connection.kind === 'webdav' ? 'WebDAV' : 'CloudDrive' }})</option></select></label>
        <label class="form-field"><span>{{ t('Source path') }}</span><input v-model="storageForm.sourcePath" required autocomplete="off" :placeholder="storageForm.sourceType === 'local' ? 'C:\\Downloads\\Anime' : '/Incoming'" /></label>
        <label class="form-field"><span>{{ t('Target type') }}</span><select v-model="storageForm.targetType"><option value="local">{{ t('Local filesystem') }}</option><option value="connection">{{ t('Saved connection') }}</option></select></label>
        <label v-if="storageForm.targetType === 'connection'" class="form-field"><span>{{ t('Target connection') }}</span><select v-model.number="storageForm.targetConnectionId" required><option :value="null" disabled>{{ t('Choose a connection') }}</option><option v-for="connection in connections" :key="connection.id" :value="connection.id">{{ connection.name }} ({{ connection.kind === 'webdav' ? 'WebDAV' : 'CloudDrive' }})</option></select></label>
        <label class="form-field"><span>{{ t('Target path') }}</span><input v-model="storageForm.targetPath" required autocomplete="off" :placeholder="storageForm.targetType === 'local' ? 'S:\\Anime' : '/Anime'" /></label>
        <label class="form-field"><span>{{ t('Mode') }}</span><select v-model="storageForm.mode"><option value="copy">{{ t('Copy') }}</option><option value="move">{{ t('Move') }}</option></select></label>
      </div>
      <div class="checkbox-grid">
        <label class="checkbox-field"><input v-model="storageForm.seasonMode" type="checkbox" /><span>{{ t('Season mode') }}</span></label>
        <label class="checkbox-field"><input v-model="storageForm.mlip" type="checkbox" /><span>{{ t('Build MLIP library') }}</span></label>
        <label class="checkbox-field"><input v-model="storageForm.removeEmptyDirs" type="checkbox" /><span>{{ t('Remove empty source folders') }}</span></label>
      </div>
    </section>
    <div v-if="storageForm.mode === 'move' || storageForm.removeEmptyDirs" class="notice danger-confirmation" role="alert">
      <strong>{{ t('Confirmation required') }}</strong>
      <label class="checkbox-field"><input v-model="storageConfirmed" type="checkbox" /><span>{{ t('I confirm the destination must verify before source files are deleted.') }}</span></label>
    </div>
    <div class="form-actions"><button class="button primary" type="submit" :disabled="storageSubmitting"><LoaderCircle v-if="storageSubmitting" class="spinning" :size="16" aria-hidden="true" /><Send v-else :size="16" aria-hidden="true" />{{ storageSubmitting ? t('Submitting...') : t('Submit cross-storage job') }}</button></div>
  </form>
</template>
