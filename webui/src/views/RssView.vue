<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { FolderSync, Pencil, Play, Power, RefreshCw, Trash2 } from 'lucide-vue-next'
import { RouterLink } from 'vue-router'
import { api, errorMessage, type Connection, type Subscription } from '../api'
import { formatDateTime, t, type MessageParams } from '../i18n'

const subscriptions = ref<Subscription[]>([])
const connections = ref<Connection[]>([])
const editing = ref<number | null>(null)
const form = ref({ url: '', filter_regex: '', target_folder: '/', interval_secs: 300, connection_id: null as number | null, auto_organize: false, organize_target_folder: '', organize_target_connection_id: null as number | null, organize_interval_secs: 300, organize_mode: 'offline' as 'offline' | 'original', organize_season_mode: true, remove_empty_dirs: true, remote_mlip: false })
const sourceConnections = computed(() => connections.value.filter(connection => connection.kind === 'clouddrive'))
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
  form.value = { url: '', filter_regex: '', target_folder: '/', interval_secs: 300, connection_id: null, auto_organize: false, organize_target_folder: '', organize_target_connection_id: null, organize_interval_secs: 300, organize_mode: 'offline', organize_season_mode: true, remove_empty_dirs: true, remote_mlip: false }
}

function edit(item: Subscription) {
  editing.value = item.id
  form.value = {
    url: item.url,
    filter_regex: item.filter_regex ?? '',
    target_folder: item.target_folder,
    interval_secs: item.interval_secs,
    connection_id: item.connection_id,
    auto_organize: item.auto_organize,
    organize_target_folder: item.organize_target_folder ?? '',
    organize_target_connection_id: item.organize_target_connection_id,
    organize_interval_secs: item.organize_interval_secs,
    organize_mode: item.organize_mode,
    organize_season_mode: item.organize_season_mode,
    remove_empty_dirs: item.remove_empty_dirs,
    remote_mlip: item.remote_mlip,
  }
}

async function save() {
  if (form.value.connection_id === null) {
    error.value = t('A CloudDrive connection is required.')
    return
  }
  if (form.value.auto_organize && !form.value.organize_target_folder.trim()) {
    error.value = t('A remote organize target is required when automatic organization is enabled.')
    return
  }
  busy.value = true
  saving.value = true
  error.value = ''
  notice.value = null
  try {
    const payload = { ...form.value, filter_regex: form.value.filter_regex || null, organize_target_folder: form.value.auto_organize ? form.value.organize_target_folder.trim() : null }
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

async function runOrganization(item: Subscription) {
  if (!window.confirm(t('Run remote organization now?'))) return
  busy.value = true
  error.value = ''
  notice.value = null
  try {
    const result = await api.runOrganization(item.id)
    notice.value = { key: 'Remote organization queued as job #{id}', params: { id: result.job.id } }
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
    <div class="table-wrap"><table><thead><tr><th>{{ t('URL') }}</th><th>{{ t('Target') }}</th><th>{{ t('Organization') }}</th><th>{{ t('Feed polling interval') }}</th><th>{{ t('Organization interval') }}</th><th>{{ t('State') }}</th><th>{{ t('Last checked') }}</th><th><span class="sr-only">{{ t('Actions') }}</span></th></tr></thead><tbody>
      <tr v-for="item in subscriptions" :key="item.id">
        <td><RouterLink :to="`/rss/${item.id}`">{{ item.url }}</RouterLink><small class="table-subtext">{{ item.filter_regex || t('No filter') }}</small></td>
        <td class="error-cell">{{ item.target_folder }}</td>
        <td><span>{{ t(item.auto_organize ? 'Automatic' : 'Off') }}</span><small v-if="item.auto_organize" class="table-subtext">{{ t(item.organize_mode === 'original' ? 'Original source mode' : 'Offline task mode') }}</small><small v-if="item.auto_organize" class="table-subtext error-cell">{{ item.organize_target_folder }}</small><small v-if="item.auto_organize" class="table-subtext">{{ t('Last organization check') }}: {{ item.last_organize_checked_at ? formatDateTime(item.last_organize_checked_at) : t('Never') }}</small></td>
        <td>{{ t('{seconds}s', { seconds: item.interval_secs }) }}</td>
        <td>{{ item.auto_organize ? t('{seconds}s', { seconds: item.organize_interval_secs }) : '-' }}</td>
        <td>{{ t(item.enabled ? 'Enabled' : 'Disabled') }}<small v-if="item.connection_id === null" class="table-subtext">{{ t('Connection required') }}</small></td>
        <td>{{ item.last_checked_at ? formatDateTime(item.last_checked_at) : t('Never') }}</td>
        <td class="actions">
          <button class="icon-button" type="button" :title="t(item.enabled ? 'Disable subscription' : 'Enable subscription')" :aria-label="t(item.enabled ? 'Disable subscription' : 'Enable subscription')" :disabled="busy || loading || (!item.enabled && item.connection_id === null)" @click="update(item)"><Power :size="15" aria-hidden="true" /></button>
          <button class="icon-button" type="button" :title="t('Run this subscription')" :aria-label="t('Run this subscription')" :disabled="busy || loading || !item.enabled || item.connection_id === null" @click="runOne(item)"><Play :size="15" aria-hidden="true" /></button>
          <button class="icon-button" type="button" :title="t('Run remote organization')" :aria-label="t('Run remote organization')" :disabled="busy || loading || !item.enabled || !item.auto_organize || item.connection_id === null" @click="runOrganization(item)"><FolderSync :size="15" aria-hidden="true" /></button>
          <button class="icon-button" type="button" :title="t('Edit subscription')" :aria-label="t('Edit subscription')" :disabled="busy || loading" @click="edit(item)"><Pencil :size="15" aria-hidden="true" /></button>
          <button class="icon-button danger-action" type="button" :title="t('Delete subscription')" :aria-label="t('Delete subscription')" :disabled="busy || loading" @click="remove(item)"><Trash2 :size="15" aria-hidden="true" /></button>
        </td>
      </tr>
      <tr v-if="!loading && !subscriptions.length"><td colspan="8" class="empty-cell">{{ t('No subscriptions.') }}</td></tr>
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
        <label class="form-field"><span>{{ t('Feed polling interval seconds') }}</span><input v-model.number="form.interval_secs" type="number" min="30" max="86400" required /></label>
        <label class="form-field"><span>{{ t('CloudDrive connection') }}</span><select v-model.number="form.connection_id" required><option :value="null" disabled>{{ t('Choose a connection') }}</option><option v-for="connection in sourceConnections" :key="connection.id" :value="connection.id">{{ connection.name }}</option></select></label>
        <label class="form-field"><span>{{ t('Organization target connection') }}</span><select v-model="form.organize_target_connection_id" :disabled="!form.auto_organize"><option :value="null">{{ t('Use source connection') }}</option><option v-for="connection in connections" :key="connection.id" :value="connection.id">{{ connection.name }} ({{ connection.kind === 'webdav' ? 'WebDAV' : 'CloudDrive' }})</option></select></label>
        <label class="form-field"><span>{{ t('Remote organize target folder') }}</span><input v-model="form.organize_target_folder" type="text" autocomplete="off" placeholder="/Anime" :disabled="!form.auto_organize" :required="form.auto_organize" /></label>
        <label class="form-field"><span>{{ t('Organization mode') }}</span><select v-model="form.organize_mode" :disabled="!form.auto_organize"><option value="offline">{{ t('Offline task mode') }}</option><option value="original">{{ t('Original source mode') }}</option></select></label>
        <label class="form-field"><span>{{ t('Organization interval seconds') }}</span><input v-model.number="form.organize_interval_secs" type="number" min="60" max="86400" :disabled="!form.auto_organize" :required="form.auto_organize" /></label>
      </div>
      <div class="checkbox-grid">
        <label class="checkbox-field"><input v-model="form.auto_organize" type="checkbox" /><span>{{ t('Automatically organize completed downloads') }}</span></label>
        <label class="checkbox-field"><input v-model="form.organize_season_mode" type="checkbox" :disabled="!form.auto_organize" /><span>{{ t('Season mode') }}</span></label>
        <label class="checkbox-field"><input v-model="form.remove_empty_dirs" type="checkbox" :disabled="!form.auto_organize" /><span>{{ t('Remove empty source folders') }}</span></label>
        <label class="checkbox-field"><input v-model="form.remote_mlip" type="checkbox" :disabled="!form.auto_organize" /><span>{{ t('Publish remote MLIP library index') }}</span></label>
      </div>
      <div class="form-actions"><button class="button secondary" type="button" :disabled="busy || loading" @click="reset">{{ t('Clear') }}</button><button class="button primary" type="submit" :disabled="busy || loading || !connections.length">{{ t(saving ? 'Saving...' : 'Save subscription') }}</button></div>
    </form>
  </section>
</template>
