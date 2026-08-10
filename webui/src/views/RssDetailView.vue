<script setup lang="ts">
import { ref, watch } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import { api, errorMessage, type DownloadTask, type ProcessedItem, type Subscription } from '../api'
import { formatDateTime, t, valueLabel } from '../i18n'

const route = useRoute()
const item = ref<Subscription | null>(null)
const processed = ref<ProcessedItem[]>([])
const tasks = ref<DownloadTask[]>([])
const error = ref('')
const loading = ref(true)
let loadVersion = 0

async function load() {
  const version = ++loadVersion
  item.value = null
  processed.value = []
  tasks.value = []
  error.value = ''
  loading.value = true
  const id = Number(route.params.id)
  if (!Number.isSafeInteger(id) || id <= 0) {
    error.value = t('Subscription not found.')
    loading.value = false
    return
  }
  try {
    const [rss, processedResult, taskResult] = await Promise.all([
      api.subscriptions(),
      api.processed(id),
      api.tasks(id),
    ])
    if (version !== loadVersion) return
    item.value = rss.subscriptions.find(value => value.id === id) ?? null
    processed.value = processedResult.items
    tasks.value = taskResult.tasks
    if (!item.value) error.value = t('Subscription not found.')
  } catch (reason) {
    if (version === loadVersion) error.value = errorMessage(reason)
  } finally {
    if (version === loadVersion) loading.value = false
  }
}

watch(() => route.params.id, load, { immediate: true })
</script>

<template>
  <div class="page-header"><div><RouterLink class="back-link" to="/rss">{{ t('Back to RSS') }}</RouterLink><h1>{{ t('Subscription details') }}</h1><p v-if="item" class="page-subtitle error-cell">{{ item.url }} / {{ item.target_folder }}</p></div></div>
  <p v-if="error" class="alert error" role="alert">{{ error }}</p>
  <p v-if="loading" class="loading-line">{{ t('Loading subscription...') }}</p>
  <template v-if="item">
    <section class="section-block"><h2>{{ t('Organization') }}</h2><div class="table-wrap"><table><thead><tr><th>{{ t('Auto organize') }}</th><th>{{ t('Remote organize target folder') }}</th><th>{{ t('Organization interval') }}</th><th>{{ t('Season mode') }}</th><th>{{ t('Remove empty source folders') }}</th><th>{{ t('Last organization check') }}</th></tr></thead><tbody><tr><td>{{ t(item.auto_organize ? 'Automatic' : 'Off') }}</td><td class="error-cell">{{ item.auto_organize ? item.organize_target_folder : '-' }}</td><td>{{ item.auto_organize ? t('{seconds}s', { seconds: item.organize_interval_secs }) : '-' }}</td><td>{{ item.auto_organize ? t(item.organize_season_mode ? 'Enabled' : 'Disabled') : '-' }}</td><td>{{ item.auto_organize ? t(item.remove_empty_dirs ? 'Enabled' : 'Disabled') : '-' }}</td><td>{{ item.auto_organize && item.last_organize_checked_at ? formatDateTime(item.last_organize_checked_at) : '-' }}</td></tr></tbody></table></div></section>
    <section class="section-block"><h2>{{ t('Processed items') }}</h2><div class="table-wrap"><table><thead><tr><th>{{ t('Title') }}</th><th>{{ t('Hash') }}</th><th>{{ t('Processed') }}</th></tr></thead><tbody>
      <tr v-for="entry in processed" :key="entry.id"><td>{{ entry.title || t('Untitled') }}</td><td class="error-cell">{{ entry.item_hash }}</td><td>{{ formatDateTime(entry.processed_at) }}</td></tr>
      <tr v-if="!processed.length"><td colspan="3" class="empty-cell">{{ t('No processed items.') }}</td></tr>
    </tbody></table></div></section>
    <section class="section-block"><h2>{{ t('Download tasks') }}</h2><div class="table-wrap"><table><thead><tr><th>{{ t('Hash') }}</th><th>{{ t('Info hash') }}</th><th>{{ t('Remote name') }}</th><th>{{ t('Cloud') }}</th><th>{{ t('Status') }}</th><th>{{ t('Added') }}</th><th>{{ t('Completed') }}</th></tr></thead><tbody>
      <tr v-for="entry in tasks" :key="entry.id"><td class="error-cell">{{ entry.item_hash }}</td><td class="error-cell">{{ entry.info_hash ?? '-' }}</td><td class="error-cell">{{ entry.remote_name ?? '-' }}</td><td>{{ entry.cloud_name ?? '-' }}</td><td>{{ entry.status ? valueLabel(entry.status) : '-' }}</td><td>{{ formatDateTime(entry.added_at) }}</td><td>{{ formatDateTime(entry.completed_at) }}</td></tr>
      <tr v-if="!tasks.length"><td colspan="7" class="empty-cell">{{ t('No download tasks.') }}</td></tr>
    </tbody></table></div></section>
  </template>
</template>
