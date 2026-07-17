<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { ChevronRight, LoaderCircle, Save, Send, Trash2 } from 'lucide-vue-next'
import { useRouter } from 'vue-router'
import { api, errorMessage } from '../api'
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

watch(needsConfirmation, (required, wasRequired) => {
  if (required !== wasRequired) confirmed.value = false
})

function errorFor(field: string) {
  return errors.value[field]
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
    presetError.value = 'Preset name is required.'
    return
  }
  presets.value = saveOrganizePreset(name, form)
  selectedPreset.value = name
  presetName.value = ''
  presetError.value = ''
}

function deletePreset() {
  if (!selectedPreset.value) return
  presets.value = deleteOrganizePreset(selectedPreset.value)
  selectedPreset.value = ''
}

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
    <div><p class="eyebrow">Manual job</p><h1>Organize</h1><p class="page-subtitle">Submit one complete organize request to the daemon queue.</p></div>
  </div>

  <p v-if="apiError" class="alert error" role="alert">{{ apiError }}</p>
  <section class="section-block preset-section" aria-labelledby="preset-heading">
    <div class="section-heading"><div><p class="eyebrow">Defaults</p><h2 id="preset-heading">Presets</h2></div></div>
    <div class="preset-bar">
      <label class="form-field preset-select"><span>Saved preset</span><select v-model="selectedPreset" @change="loadPreset"><option value="">Choose a preset</option><option v-for="preset in presets" :key="preset.name" :value="preset.name">{{ preset.name }}</option></select></label>
      <button class="button secondary" type="button" :disabled="!selectedPreset" @click="loadPreset"><ChevronRight :size="16" aria-hidden="true" />Load</button>
      <button class="icon-button danger-action" type="button" title="Delete selected preset" aria-label="Delete selected preset" :disabled="!selectedPreset" @click="deletePreset"><Trash2 :size="16" aria-hidden="true" /></button>
      <label class="form-field preset-name"><span>New preset name</span><input v-model="presetName" type="text" maxlength="80" placeholder="Optional saved name" @keyup.enter="savePreset" /><small v-if="presetError" class="field-error" role="alert">{{ presetError }}</small></label>
      <button class="button secondary" type="button" @click="savePreset"><Save :size="16" aria-hidden="true" />Save preset</button>
    </div>
  </section>

  <form class="organize-form" novalidate @submit.prevent="submit">
    <section class="section-block" aria-labelledby="paths-heading">
      <div class="section-heading"><div><p class="eyebrow">Required</p><h2 id="paths-heading">Paths and operation</h2></div></div>
      <div class="form-grid">
        <label class="form-field"><span>Source <b aria-hidden="true">*</b></span><input v-model="form.source" type="text" autocomplete="off" placeholder="C:\\Downloads\\Anime" :aria-invalid="Boolean(errorFor('source'))" aria-describedby="source-error" /><small v-if="errorFor('source')" id="source-error" class="field-error" role="alert">{{ errorFor('source') }}</small></label>
        <label class="form-field"><span>Target <b aria-hidden="true">*</b></span><input v-model="form.target" type="text" autocomplete="off" placeholder="S:\\Anime" :aria-invalid="Boolean(errorFor('target'))" aria-describedby="target-error" /><small v-if="errorFor('target')" id="target-error" class="field-error" role="alert">{{ errorFor('target') }}</small></label>
        <label class="form-field"><span>Mode</span><select v-model="form.mode" :aria-invalid="Boolean(errorFor('mode'))"><option value="link">Hard link</option><option value="copy">Copy</option><option value="move">Move</option></select><small v-if="errorFor('mode')" class="field-error" role="alert">{{ errorFor('mode') }}</small></label>
        <label class="form-field"><span>Link failure fallback</span><select v-model="form.fallback_on_link_failure"><option value="">No fallback</option><option value="move">Move</option><option value="copy">Copy</option></select><small v-if="errorFor('fallback_on_link_failure')" class="field-error" role="alert">{{ errorFor('fallback_on_link_failure') }}</small></label>
      </div>
      <label class="checkbox-field"><input v-model="form.dry_run" type="checkbox" /><span>Dry run</span></label>
    </section>

    <details class="section-block advanced-options">
      <summary><span><span class="eyebrow">Optional</span><strong>Advanced options</strong></span></summary>
      <div class="form-grid">
        <label class="form-field"><span>Included extensions</span><input v-model="form.include_ext" type="text" placeholder="mp4,mkv,avi" :aria-invalid="Boolean(errorFor('include_ext'))" /><small v-if="errorFor('include_ext')" class="field-error" role="alert">{{ errorFor('include_ext') }}</small></label>
        <label class="form-field"><span>Filename parser</span><select v-model="form.filename_parser" :aria-invalid="Boolean(errorFor('filename_parser'))"><option value="rules">Rules</option><option value="anifilebert">AniFileBERT</option><option value="auto">Auto</option></select><small v-if="errorFor('filename_parser')" class="field-error" role="alert">{{ errorFor('filename_parser') }}</small></label>
        <label class="form-field"><span>TMDB API key</span><input v-model="form.tmdb_api_key" type="password" autocomplete="off" placeholder="Optional key" :aria-invalid="Boolean(errorFor('tmdb_api_key'))" /><small v-if="errorFor('tmdb_api_key')" class="field-error" role="alert">{{ errorFor('tmdb_api_key') }}</small></label>
        <label class="form-field"><span>Alias file</span><input v-model="form.alias_file" type="text" autocomplete="off" placeholder="Optional JSON path" :aria-invalid="Boolean(errorFor('alias_file'))" /><small v-if="errorFor('alias_file')" class="field-error" role="alert">{{ errorFor('alias_file') }}</small></label>
        <label class="form-field"><span>Bangumi cache</span><input v-model="form.bangumi_cache" type="text" autocomplete="off" placeholder="Optional cache path" :aria-invalid="Boolean(errorFor('bangumi_cache'))" /><small v-if="errorFor('bangumi_cache')" class="field-error" role="alert">{{ errorFor('bangumi_cache') }}</small></label>
        <label class="form-field"><span>Metadata source</span><input v-model="form.metadata_source" type="text" autocomplete="off" placeholder="Optional local subject.jsonlines path" :aria-invalid="Boolean(errorFor('metadata_source'))" /><small v-if="errorFor('metadata_source')" class="field-error" role="alert">{{ errorFor('metadata_source') }}</small></label>
      </div>
      <div class="checkbox-grid">
        <label class="checkbox-field"><input v-model="form.verbose" type="checkbox" /><span>Verbose logging</span></label>
        <label class="checkbox-field"><input v-model="form.scrape_metadata" type="checkbox" /><span>Scrape metadata</span></label>
        <label class="checkbox-field"><input v-model="form.no_images" type="checkbox" /><span>Skip images</span></label>
        <label class="checkbox-field"><input v-model="form.no_episode_metadata" type="checkbox" /><span>Skip episode metadata</span></label>
        <label class="checkbox-field"><input v-model="form.force_overwrite" type="checkbox" /><span>Overwrite existing metadata</span></label>
        <label class="checkbox-field"><input v-model="form.season_mode" type="checkbox" /><span>Season mode</span></label>
        <label class="checkbox-field"><input v-model="form.library_index" type="checkbox" /><span>Update library index</span></label>
        <label class="checkbox-field"><input v-model="form.mlip" type="checkbox" /><span>Build MLIP library</span></label>
        <label class="checkbox-field"><input v-model="form.rebuild_library_index" type="checkbox" :aria-invalid="Boolean(errorFor('rebuild_library_index'))" /><span>Rebuild library index<small v-if="errorFor('rebuild_library_index')" class="field-error" role="alert">{{ errorFor('rebuild_library_index') }}</small></span></label>
        <label class="checkbox-field"><input v-model="form.probe_runtime" type="checkbox" /><span>Probe runtime with ffprobe</span></label>
      </div>
    </details>

    <div v-if="needsConfirmation" class="notice danger-confirmation" role="alert">
      <strong>Confirmation required</strong>
      <label class="checkbox-field"><input v-model="confirmed" type="checkbox" :aria-invalid="Boolean(errorFor('confirmed'))" /><span>I understand this request can change files or rebuild the library index.</span></label>
      <small v-if="errorFor('confirmed')" class="field-error" role="alert">{{ errorFor('confirmed') }}</small>
    </div>
    <div class="form-actions"><button class="button primary" type="submit" :disabled="submitting"><LoaderCircle v-if="submitting" class="spinning" :size="16" aria-hidden="true" /><Send v-else :size="16" aria-hidden="true" />{{ submitting ? 'Submitting...' : 'Submit organize job' }}</button><span class="form-hint">Required fields are marked *</span></div>
  </form>
</template>
