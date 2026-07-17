import type { OrganizeArgs } from './api'

export interface OrganizeForm {
  source: string
  target: string
  mode: OrganizeArgs['mode']
  fallback_on_link_failure: NonNullable<OrganizeArgs['fallback_on_link_failure']> | ''
  dry_run: boolean
  include_ext: string
  verbose: boolean
  scrape_metadata: boolean
  tmdb_api_key: string
  alias_file: string
  no_images: boolean
  no_episode_metadata: boolean
  force_overwrite: boolean
  bangumi_cache: string
  metadata_source: string
  season_mode: boolean
  library_index: boolean
  mlip: boolean
  rebuild_library_index: boolean
  probe_runtime: boolean
  filename_parser: OrganizeArgs['filename_parser']
}

type PresetForm = Omit<OrganizeForm, 'tmdb_api_key'>

export interface OrganizePreset {
  name: string
  form: PresetForm
}

export const organizePresetStorageKey = 'anime-organizer.organize-presets'

export function defaultOrganizeForm(): OrganizeForm {
  return {
    source: '', target: '', mode: 'link', fallback_on_link_failure: '', dry_run: false, include_ext: 'mp4,mkv,avi,mov,wmv,flv,rmvb',
    verbose: false, scrape_metadata: false, tmdb_api_key: '', alias_file: '', no_images: false, no_episode_metadata: false,
    force_overwrite: false, bangumi_cache: '', metadata_source: '', season_mode: false, library_index: false, mlip: false,
    rebuild_library_index: false, probe_runtime: false, filename_parser: 'rules',
  }
}

export function toOrganizeArgs(form: OrganizeForm): OrganizeArgs {
  const optional = (value: string): string | null => value.trim() || null
  const includeExt = form.include_ext.split(',').map((item) => item.trim()).filter(Boolean)
  return {
    source: optional(form.source), target: optional(form.target), mode: form.mode,
    fallback_on_link_failure: form.fallback_on_link_failure || null, dry_run: form.dry_run,
    include_ext: includeExt.length ? includeExt : null, verbose: form.verbose, scrape_metadata: form.scrape_metadata,
    tmdb_api_key: optional(form.tmdb_api_key), alias_file: optional(form.alias_file), no_images: form.no_images,
    no_episode_metadata: form.no_episode_metadata, force_overwrite: form.force_overwrite, bangumi_cache: optional(form.bangumi_cache),
    metadata_source: optional(form.metadata_source), season_mode: form.season_mode, library_index: form.library_index,
    mlip: form.mlip, rebuild_library_index: form.rebuild_library_index, probe_runtime: form.probe_runtime,
    filename_parser: form.filename_parser,
  }
}

export function validateOrganize(form: OrganizeForm, confirmed = false): Record<string, string> {
  const errors: Record<string, string> = {}
  if (!form.source.trim()) errors.source = 'Source is required.'
  if (!form.target.trim()) errors.target = 'Target is required.'
  if (form.rebuild_library_index && !form.library_index && !form.mlip) errors.rebuild_library_index = 'Enable library index or MLIP before rebuilding.'
  if ((form.mode === 'move' || form.rebuild_library_index) && !confirmed) errors.confirmed = 'Confirm this move or index rebuild before submitting.'
  return errors
}

export function fieldErrors(message: string): Record<string, string> {
  const errors: Record<string, string> = {}
  const lowerMessage = message.toLowerCase()
  for (const field of [
    'source', 'target', 'mode', 'fallback_on_link_failure', 'include_ext', 'tmdb_api_key', 'alias_file',
    'bangumi_cache', 'metadata_source', 'library_index', 'mlip', 'rebuild_library_index', 'filename_parser', 'confirmed',
  ]) {
    if (lowerMessage.includes(field)) errors[field] = message
  }
  if (lowerMessage.includes('confirm')) errors.confirmed = message
  return errors
}

function storage(): Storage | undefined {
  try { return typeof window === 'undefined' ? undefined : window.localStorage } catch { return undefined }
}

function withoutSecret(form: OrganizeForm): PresetForm {
  const { tmdb_api_key: _tmdbApiKey, ...safeForm } = form
  return safeForm
}

export function loadOrganizePresets(target = storage()): OrganizePreset[] {
  if (!target) return []
  try {
    const value: unknown = JSON.parse(target.getItem(organizePresetStorageKey) ?? '[]')
    if (!Array.isArray(value)) return []
    return value.flatMap((item): OrganizePreset[] => {
      if (!item || typeof item !== 'object') return []
      const record = item as Record<string, unknown>
      if (typeof record.name !== 'string' || !record.name.trim() || !record.form || typeof record.form !== 'object') return []
      const { tmdb_api_key: _ignored, ...safeForm } = record.form as Record<string, unknown>
      return [{ name: record.name, form: { ...withoutSecret(defaultOrganizeForm()), ...(safeForm as Partial<PresetForm>) } }]
    })
  } catch { return [] }
}

export function saveOrganizePreset(name: string, form: OrganizeForm, target = storage()): OrganizePreset[] {
  const presets = loadOrganizePresets(target)
  const preset = { name: name.trim(), form: withoutSecret(form) }
  const next = [...presets.filter((item) => item.name !== preset.name), preset]
  try { target?.setItem(organizePresetStorageKey, JSON.stringify(next)) } catch { /* Storage may be unavailable or full. */ }
  return next
}

export function deleteOrganizePreset(name: string, target = storage()): OrganizePreset[] {
  const next = loadOrganizePresets(target).filter((item) => item.name !== name)
  try { target?.setItem(organizePresetStorageKey, JSON.stringify(next)) } catch { /* Storage may be unavailable or full. */ }
  return next
}
