<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { ChevronRight, Download, Folder, RefreshCw, TestTube, Trash2 } from 'lucide-vue-next'
import { api, errorMessage, type Connection, type FolderEntry } from '../api'
import { breadcrumbEntries } from '../cloud'
import { t } from '../i18n'

const connections = ref<Connection[]>([])
const selected = ref<Connection | null>(null)
const path = ref('/')
const breadcrumbs = computed(() => breadcrumbEntries(path.value))
const entries = ref<FolderEntry[]>([])
const error = ref('')
const notice = ref('')
const loading = ref(false)
const busy = ref(false)
const saving = ref(false)
const form = ref({ name: '', url: '', token: '', username: '', password: '' })
const offline = ref({ url: '', target: '/' })
const queuedJobId = ref<number | null>(null)

function hasCredentials(connection: Connection) {
  return connection.has_token || (connection.has_username && connection.has_password)
}

async function load() {
  loading.value = true
  try {
    connections.value = (await api.connections()).connections
    error.value = ''
  } catch (reason) {
    error.value = errorMessage(reason)
  } finally {
    loading.value = false
  }
}

async function save() {
  if (!form.value.token && (!form.value.username || !form.value.password)) {
    error.value = t('A token or username/password login is required.')
    return
  }
  if (Boolean(form.value.username) !== Boolean(form.value.password)) {
    error.value = t('Username and password must be provided together.')
    return
  }
  busy.value = true
  saving.value = true
  error.value = ''
  notice.value = ''
  try {
    await api.saveConnection({
      ...form.value,
      token: form.value.token || null,
      username: form.value.username || null,
      password: form.value.password || null,
    })
    form.value = { name: '', url: '', token: '', username: '', password: '' }
    notice.value = 'Connection saved.'
    await load()
  } catch (reason) {
    error.value = errorMessage(reason)
  } finally {
    busy.value = false
    saving.value = false
  }
}

async function test(connection: Connection) {
  busy.value = true
  error.value = ''
  notice.value = ''
  try {
    await api.testConnection(connection.id)
    notice.value = 'Connection test succeeded.'
  } catch (reason) {
    error.value = errorMessage(reason)
  } finally {
    busy.value = false
  }
}

async function remove(connection: Connection) {
  if (!window.confirm(t('Delete this connection?'))) return
  busy.value = true
  error.value = ''
  notice.value = ''
  try {
    await api.deleteConnection(connection.id)
    if (selected.value?.id === connection.id) {
      selected.value = null
      entries.value = []
    }
    notice.value = 'Connection deleted.'
    await load()
  } catch (reason) {
    error.value = errorMessage(reason)
  } finally {
    busy.value = false
  }
}

async function select(connection: Connection) {
  selected.value = connection
  path.value = '/'
  offline.value.target = '/'
  queuedJobId.value = null
  await browse('/')
}

async function browse(nextPath: string) {
  if (!selected.value) return
  busy.value = true
  error.value = ''
  notice.value = ''
  entries.value = []
  path.value = nextPath
  try {
    entries.value = (await api.listFolder(selected.value.id, nextPath)).entries
  } catch (reason) {
    error.value = errorMessage(reason)
  } finally {
    busy.value = false
  }
}

async function submitOffline() {
  if (!selected.value || !window.confirm(t('Queue this offline URL?'))) return
  busy.value = true
  error.value = ''
  queuedJobId.value = null
  try {
    const result = await api.enqueueCloudAddOffline({ connection_id: selected.value.id, ...offline.value })
    queuedJobId.value = result.job.id
    offline.value.url = ''
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
    <div><p class="eyebrow">{{ t('Saved credentials') }}</p><h1>{{ t('CloudDrive connections') }}</h1><p class="page-subtitle">{{ t('Credentials are stored by the local daemon and shown here only as presence flags.') }}</p></div>
    <button class="button secondary" type="button" :disabled="loading || busy" @click="load"><RefreshCw :size="15" :class="{ spinning: loading }" aria-hidden="true" />{{ t('Refresh') }}</button>
  </div>
  <p v-if="error" class="alert error" role="alert">{{ error }}</p>
  <p v-if="notice" class="alert success-text" role="status">{{ t(notice) }}</p>

  <section class="section-block"><div class="table-wrap"><table><thead><tr><th>{{ t('Name') }}</th><th>{{ t('Endpoint') }}</th><th>{{ t('Credentials') }}</th><th><span class="sr-only">{{ t('Actions') }}</span></th></tr></thead><tbody>
    <tr v-for="connection in connections" :key="connection.id">
      <td>{{ connection.name }}</td><td class="error-cell">{{ connection.url }}</td>
      <td>{{ connection.has_token ? t('Token set') : connection.has_username && connection.has_password ? t('Login set') : t('No credentials') }}</td>
      <td class="actions">
        <button class="icon-button" type="button" :title="t('Browse folder')" :aria-label="t('Browse folder')" :disabled="busy || loading || !hasCredentials(connection)" @click="select(connection)"><Folder :size="15" aria-hidden="true" /></button>
        <button class="icon-button" type="button" :title="t('Test connection')" :aria-label="t('Test connection')" :disabled="busy || loading || !hasCredentials(connection)" @click="test(connection)"><TestTube :size="15" aria-hidden="true" /></button>
        <button class="icon-button danger-action" type="button" :title="t('Delete connection')" :aria-label="t('Delete connection')" :disabled="busy || loading" @click="remove(connection)"><Trash2 :size="15" aria-hidden="true" /></button>
      </td>
    </tr>
    <tr v-if="!loading && !connections.length"><td colspan="4" class="empty-cell">{{ t('No connections.') }}</td></tr>
  </tbody></table></div><p v-if="loading" class="loading-line">{{ t('Loading...') }}</p></section>

  <section class="section-block"><h2>{{ t('Add connection') }}</h2><form class="organize-form" @submit.prevent="save"><div class="form-grid">
    <label class="form-field"><span>{{ t('Name') }}</span><input v-model="form.name" required /></label>
    <label class="form-field"><span>{{ t('Endpoint') }}</span><input v-model="form.url" type="url" required /></label>
    <label class="form-field"><span>{{ t('Token') }}</span><input v-model="form.token" type="password" autocomplete="off" /></label>
    <label class="form-field"><span>{{ t('Username') }}</span><input v-model="form.username" autocomplete="username" /></label>
    <label class="form-field"><span>{{ t('Password') }}</span><input v-model="form.password" type="password" autocomplete="current-password" /></label>
  </div><div class="form-actions"><button class="button primary" type="submit" :disabled="busy || loading">{{ t(saving ? 'Saving...' : 'Save connection') }}</button></div></form></section>

  <section v-if="selected" class="section-block">
    <h2>{{ t('Folder browser / {name}', { name: selected.name }) }}</h2>
    <div class="breadcrumbs"><button type="button" :disabled="busy" @click="browse('/')">{{ t('Root') }}</button><template v-for="breadcrumb in breadcrumbs" :key="breadcrumb.path"><ChevronRight :size="15" aria-hidden="true" /><button type="button" :disabled="busy" @click="browse(breadcrumb.path)">{{ breadcrumb.label }}</button></template></div>
    <form class="organize-form" @submit.prevent="submitOffline"><div class="form-grid"><label class="form-field"><span>{{ t('Offline URL') }}</span><input v-model="offline.url" required /></label><label class="form-field"><span>{{ t('Target folder') }}</span><input v-model="offline.target" required /></label></div><div class="form-actions"><span v-if="queuedJobId" class="success-text" role="status">{{ t('Queued job {id}', { id: queuedJobId }) }}</span><button class="button primary" type="submit" :disabled="busy || loading"><Download :size="15" aria-hidden="true" />{{ t('Queue URL') }}</button></div></form>
    <div class="table-wrap"><table><thead><tr><th>{{ t('Name') }}</th><th>{{ t('Path') }}</th><th>{{ t('Type') }}</th><th>{{ t('Size') }}</th></tr></thead><tbody>
      <tr v-for="entry in entries" :key="entry.id"><td><button v-if="entry.is_directory" type="button" :disabled="busy" @click="browse(entry.path)">{{ entry.name }}</button><span v-else>{{ entry.name }}</span></td><td class="error-cell">{{ entry.path }}</td><td>{{ t(entry.is_directory ? 'Folder' : 'File') }}</td><td>{{ entry.size }}</td></tr>
      <tr v-if="!busy && !entries.length"><td colspan="4" class="empty-cell">{{ t('Folder is empty.') }}</td></tr>
    </tbody></table></div>
  </section>
</template>
