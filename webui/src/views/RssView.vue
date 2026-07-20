<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { Pencil, Play, Power, RefreshCw, Trash2 } from 'lucide-vue-next'
import { RouterLink } from 'vue-router'
import { api, errorMessage, type Connection, type Subscription } from '../api'
import { formatDateTime, t, type MessageParams } from '../i18n'

const subscriptions = ref<Subscription[]>([])
const connections = ref<Connection[]>([])
const editing = ref<number | null>(null)
const form = ref({ url: '', filter_regex: '', target_folder: '/', interval_secs: 300, connection_id: null as number | null })
const error = ref('')
const notice = ref<{ key: string; params?: MessageParams } | null>(null)
const loading = ref(false)
const busy = ref(false)
const saving = ref(false)

async function load() {
  loading.value = true
  try {
    const [rss, cloud] = await Promise.all([api.subscriptions(), api.connections()])
    subscriptions.value = rss.subscriptions
    connections.value = cloud.connections
    error.value = ''
  } catch (reason) {
    error.value = errorMessage(reason)
  } finally {
    loading.value = false
  }
}

function reset() {
  editing.value = null
  form.value = { url: '', filter_regex: '', target_folder: '/', interval_secs: 300, connection_id: null }
}

function edit(item: Subscription) {
  editing.value = item.id
  form.value = {
    url: item.url,
    filter_regex: item.filter_regex ?? '',
    target_folder: item.target_folder,
    interval_secs: item.interval_secs,
    connection_id: item.connection_id,
  }
}

async function save() {
  if (form.value.connection_id === null) {
    error.value = t('A CloudDrive connection is required.')
    return
  }
  busy.value = true
  saving.value = true
  error.value = ''
  notice.value = null
  try {
    const payload = { ...form.value, filter_regex: form.value.filter_regex || null }
    if (editing.value === null) await api.createSubscription(payload)
    else await api.updateSubscription(editing.value, payload)
    reset()
    notice.value = { key: 'Subscription saved.' }
    await load()
  } catch (reason) {
    error.value = errorMessage(reason)
  } finally {
    busy.value = false
    saving.value = false
  }
}

async function update(item: Subscription) {
  busy.value = true
  error.value = ''
  notice.value = null
  try {
    await api.setEnabled(item.id, !item.enabled)
    notice.value = { key: 'Subscription updated.' }
    await load()
  } catch (reason) {
    error.value = errorMessage(reason)
  } finally {
    busy.value = false
  }
}

async function runOne(item: Subscription) {
  if (!window.confirm(t('Run this subscription now?'))) return
  busy.value = true
  error.value = ''
  notice.value = null
  try {
    const result = await api.runSubscription(item.id)
    notice.value = { key: 'Subscription queued as job #{id}', params: { id: result.job.id } }
    await load()
  } catch (reason) {
    error.value = errorMessage(reason)
  } finally {
    busy.value = false
  }
}

async function runAll() {
  if (!window.confirm(t('Run all subscriptions now?'))) return
  busy.value = true
  error.value = ''
  notice.value = null
  try {
    const result = await api.runAll()
    notice.value = { key: 'Subscription queued as job #{id}', params: { id: result.job.id } }
    await load()
  } catch (reason) {
    error.value = errorMessage(reason)
  } finally {
    busy.value = false
  }
}

async function remove(item: Subscription) {
  if (!window.confirm(t('Delete this subscription?'))) return
  busy.value = true
  error.value = ''
  notice.value = null
  try {
    await api.deleteSubscription(item.id)
    if (editing.value === item.id) reset()
    notice.value = { key: 'Subscription deleted.' }
    await load()
  } catch (reason) {
    error.value = errorMessage(reason)
  } finally {
    busy.value = false
  }
}

onMounted(load)
</script>

<template>
  <div class="page-header">
    <div><p class="eyebrow">{{ t('CloudDrive feed intake') }}</p><h1>{{ t('RSS subscriptions') }}</h1><p class="page-subtitle">{{ t('Manage polling intervals, filters, destinations, and download history.') }}</p></div>
    <div class="detail-actions">
      <button class="button secondary" type="button" :title="t('Refresh subscriptions')" :disabled="loading || busy" @click="load"><RefreshCw :size="15" :class="{ spinning: loading }" aria-hidden="true" />{{ t('Refresh') }}</button>
      <button class="button primary" type="button" :disabled="busy || loading" @click="reset">{{ t('New subscription') }}</button>
    </div>
  </div>
  <p v-if="error" class="alert error" role="alert">{{ error }}</p>
  <p v-if="notice" class="alert success-text" role="status">{{ t(notice.key, notice.params) }}</p>

  <section class="section-block">
    <div class="section-heading"><h2>{{ t('Saved subscriptions') }}</h2><button class="button secondary" type="button" :disabled="busy || loading || !subscriptions.length || subscriptions.some(item => item.enabled && item.connection_id === null)" @click="runAll"><Play :size="15" aria-hidden="true" />{{ t('Run all') }}</button></div>
    <div class="table-wrap"><table><thead><tr><th>{{ t('URL') }}</th><th>{{ t('Target') }}</th><th>{{ t('Interval') }}</th><th>{{ t('State') }}</th><th>{{ t('Last checked') }}</th><th><span class="sr-only">{{ t('Actions') }}</span></th></tr></thead><tbody>
      <tr v-for="item in subscriptions" :key="item.id">
        <td><RouterLink :to="`/rss/${item.id}`">{{ item.url }}</RouterLink><small class="table-subtext">{{ item.filter_regex || t('No filter') }}</small></td>
        <td class="error-cell">{{ item.target_folder }}</td>
        <td>{{ t('{seconds}s', { seconds: item.interval_secs }) }}</td>
        <td>{{ t(item.enabled ? 'Enabled' : 'Disabled') }}<small v-if="item.connection_id === null" class="table-subtext">{{ t('Connection required') }}</small></td>
        <td>{{ item.last_checked_at ? formatDateTime(item.last_checked_at) : t('Never') }}</td>
        <td class="actions">
          <button class="icon-button" type="button" :title="t(item.enabled ? 'Disable subscription' : 'Enable subscription')" :aria-label="t(item.enabled ? 'Disable subscription' : 'Enable subscription')" :disabled="busy || loading || (!item.enabled && item.connection_id === null)" @click="update(item)"><Power :size="15" aria-hidden="true" /></button>
          <button class="icon-button" type="button" :title="t('Run this subscription')" :aria-label="t('Run this subscription')" :disabled="busy || loading || !item.enabled || item.connection_id === null" @click="runOne(item)"><Play :size="15" aria-hidden="true" /></button>
          <button class="icon-button" type="button" :title="t('Edit subscription')" :aria-label="t('Edit subscription')" :disabled="busy || loading" @click="edit(item)"><Pencil :size="15" aria-hidden="true" /></button>
          <button class="icon-button danger-action" type="button" :title="t('Delete subscription')" :aria-label="t('Delete subscription')" :disabled="busy || loading" @click="remove(item)"><Trash2 :size="15" aria-hidden="true" /></button>
        </td>
      </tr>
      <tr v-if="!loading && !subscriptions.length"><td colspan="6" class="empty-cell">{{ t('No subscriptions.') }}</td></tr>
    </tbody></table></div>
    <p v-if="loading" class="loading-line">{{ t('Loading...') }}</p>
  </section>

  <section class="section-block" aria-labelledby="rss-form-heading">
    <h2 id="rss-form-heading">{{ t(editing ? 'Edit subscription' : 'New subscription') }}</h2>
    <form class="organize-form" @submit.prevent="save">
      <div class="form-grid">
        <label class="form-field"><span>{{ t('RSS URL') }}</span><input v-model="form.url" type="url" required /></label>
        <label class="form-field"><span>{{ t('Target folder') }}</span><input v-model="form.target_folder" required /></label>
        <label class="form-field"><span>{{ t('Filter regex') }}</span><input v-model="form.filter_regex" /></label>
        <label class="form-field"><span>{{ t('Interval seconds') }}</span><input v-model.number="form.interval_secs" type="number" min="30" max="86400" required /></label>
        <label class="form-field"><span>{{ t('CloudDrive connection') }}</span><select v-model.number="form.connection_id" required><option :value="null" disabled>{{ t('Choose a connection') }}</option><option v-for="connection in connections" :key="connection.id" :value="connection.id">{{ connection.name }}</option></select></label>
      </div>
      <div class="form-actions"><button class="button secondary" type="button" :disabled="busy || loading" @click="reset">{{ t('Clear') }}</button><button class="button primary" type="submit" :disabled="busy || loading || !connections.length">{{ t(saving ? 'Saving...' : 'Save subscription') }}</button></div>
    </form>
  </section>
</template>
