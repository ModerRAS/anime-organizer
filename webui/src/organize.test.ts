import { afterEach, describe, expect, it } from 'vitest'
import {
  defaultOrganizeForm,
  fieldErrors,
  loadOrganizePresets,
  saveOrganizePreset,
  toOrganizeArgs,
  validateOrganize,
} from './organize'


afterEach(() => window.localStorage.clear())

describe('organize form contract', () => {
  it('creates the Rust snake_case request and omits empty optional values', () => {
    const form = defaultOrganizeForm()
    form.source = 'C:/downloads'; form.target = 'S:/anime'; form.mode = 'copy'; form.include_ext = 'mkv, mp4'
    const args = toOrganizeArgs(form)
    expect(args).toMatchObject({ source: 'C:/downloads', target: 'S:/anime', mode: 'copy', include_ext: ['mkv', 'mp4'], tmdb_api_key: null })
  })

  it('maps every organize field to the Rust serde contract', () => {
    const form = defaultOrganizeForm()
    Object.assign(form, {
      source: 'source', target: 'target', fallback_on_link_failure: 'move', dry_run: true, verbose: true,
      scrape_metadata: true, tmdb_api_key: 'secret', alias_file: 'aliases.json', no_images: true,
      no_episode_metadata: true, force_overwrite: true, bangumi_cache: 'cache', metadata_source: 'subjects.jsonl',
      season_mode: true, library_index: true, mlip: true, rebuild_library_index: true, probe_runtime: true,
      filename_parser: 'auto', include_ext: 'mkv, mp4', mode: 'copy',
    })
    expect(toOrganizeArgs(form)).toEqual({
      source: 'source', target: 'target', mode: 'copy', fallback_on_link_failure: 'move', dry_run: true,
      include_ext: ['mkv', 'mp4'], verbose: true, scrape_metadata: true, tmdb_api_key: 'secret', alias_file: 'aliases.json',
      no_images: true, no_episode_metadata: true, force_overwrite: true, bangumi_cache: 'cache', metadata_source: 'subjects.jsonl',
      season_mode: true, library_index: true, mlip: true, rebuild_library_index: true, probe_runtime: true, filename_parser: 'auto',
    })
  })

  it('requires confirmation for move and index rebuild operations', () => {
    const form = defaultOrganizeForm()
    form.source = 'source'; form.target = 'target'; form.mode = 'move'
    expect(validateOrganize(form)).toMatchObject({ confirmed: expect.any(String) })
    expect(validateOrganize(form, true)).not.toHaveProperty('confirmed')
  })

  it('reports required fields beside their controls and maps API fields', () => {
    expect(validateOrganize(defaultOrganizeForm())).toMatchObject({ source: 'Source is required.', target: 'Target is required.' })
    expect(fieldErrors('target is required')).toEqual({ target: 'target is required' })
    expect(fieldErrors('filename_parser anifilebert requires the feature')).toHaveProperty('filename_parser')
  })

  it('stores presets without the TMDB API key', () => {
    const form = defaultOrganizeForm()
    form.source = 'source'; form.target = 'target'; form.tmdb_api_key = 'secret'
    saveOrganizePreset('library', form)
    const raw = window.localStorage.getItem('anime-organizer.organize-presets') ?? ''
    expect(raw).not.toContain('secret')
    expect(raw).not.toContain('tmdb_api_key')
    expect(loadOrganizePresets()).toMatchObject([{ name: 'library', form: { source: 'source', target: 'target' } }])
  })
})
