import { afterEach, describe, expect, it } from 'vitest'
import { formatDateTime, locale, setLocale, t, valueLabel } from './i18n'

afterEach(() => setLocale('en'))

describe('i18n', () => {
  it('switches and persists Chinese', () => {
    setLocale('zh-CN')
    expect(locale.value).toBe('zh-CN')
    expect(t('Jobs')).toBe('任务')
    expect(t('{count} records', { count: 3 })).toBe('3 条记录')
    expect(localStorage.getItem('anime-organizer.locale')).toBe('zh-CN')
    expect(document.documentElement.lang).toBe('zh-CN')
  })

  it('falls back to readable enum values', () => {
    setLocale('zh-CN')
    expect(valueLabel('cloud_add_offline')).toBe('CloudDrive 离线下载')
    expect(valueLabel('future_job_kind')).toBe('future job kind')
  })

  it('formats API and SQLite timestamps in the active locale', () => {
    setLocale('en')
    expect(formatDateTime('1970-01-01T00:00:00Z')).not.toBe('-')
    expect(formatDateTime('1970-01-01 00:00:00')).not.toBe('-')
    expect(formatDateTime('0')).not.toBe('-')
    expect(formatDateTime(null)).toBe('-')
    expect(formatDateTime('invalid')).toBe('-')
  })
})
