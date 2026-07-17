import type { JobState } from './api'

export const jobStates: Array<{ value: '' | JobState; label: string }> = [
  { value: '', label: 'All states' }, { value: 'queued', label: 'Queued' }, { value: 'running', label: 'Running' },
  { value: 'succeeded', label: 'Succeeded' }, { value: 'failed', label: 'Failed' }, { value: 'canceled', label: 'Canceled' },
]

export function jobQueryState(value: unknown): JobState | undefined {
  return typeof value === 'string' && jobStates.some((item) => item.value === value && item.value !== '') ? value as JobState : undefined
}
