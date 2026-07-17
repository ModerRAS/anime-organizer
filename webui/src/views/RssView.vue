<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { RefreshCw, Play, Pencil } from 'lucide-vue-next'
import { api, errorMessage, type Connection, type Subscription } from '../api'

const subscriptions = ref<Subscription[]>([])
const connections = ref<Connection[]>([])
const error = ref('')
const editing = ref<number | null>(null)
const form = ref({ url: '', filter_regex: '', target_folder: '/', interval_secs: 300, connection_id: null as number | null })
async function load() { try { const [rss, cloud] = await Promise.all([api.subscriptions(), api.connections()]); subscriptions.value = rss.subscriptions; connections.value = cloud.connections; error.value = '' } catch (e) { error.value = errorMessage(e) } }
function edit(item: Subscription) { editing.value = item.id; form.value = { url: item.url, filter_regex: item.filter_regex || '', target_folder: item.target_folder, interval_secs: item.interval_secs, connection_id: item.connection_id } }
function reset() { editing.value = null; form.value = { url: '', filter_regex: '', target_folder: '/', interval_secs: 300, connection_id: null } }
async function save() { try { const value = { ...form.value, filter_regex: form.value.filter_regex || null }; if (editing.value) await api.updateSubscription(editing.value, value); else await api.createSubscription(value); reset(); await load() } catch (e) { error.value = errorMessage(e) } }
async function action(fn: () => Promise<unknown>) { try { await fn(); await load() } catch (e) { error.value = errorMessage(e) } }
onMounted(load)
</script>
<template>
  <div class="page-header"><div><p class="eyebrow">CloudDrive feed intake</p><h1>RSS subscriptions</h1><p class="page-subtitle">Manage polling intervals, filters, destinations, and download history.</p></div><div class="detail-actions"><button class="button secondary" type="button" title="Refresh subscriptions" @click="load"><RefreshCw :size="15" />Refresh</button><button class="button primary" type="button" @click="reset">New subscription</button></div></div>
  <p v-if="error" class="alert error" role="alert">{{ error }}</p>
  <section class="section-block"><div class="section-heading"><h2>Saved subscriptions</h2><button class="button secondary" type="button" @click="action(api.runAll)"><Play :size="15" />Run all</button></div><div class="table-wrap"><table><thead><tr><th>URL</th><th>Target</th><th>Interval</th><th>State</th><th>Last checked</th><th></th></tr></thead><tbody><tr v-for="item in subscriptions" :key="item.id"><td><RouterLink :to="`/rss/${item.id}`">{{ item.url }}</RouterLink><small class="table-subtext">{{ item.filter_regex || 'No filter' }}</small></td><td class="error-cell">{{ item.target_folder }}</td><td>{{ item.interval_secs }}s</td><td>{{ item.enabled ? 'Enabled' : 'Disabled' }}</td><td>{{ item.last_checked_at || 'Never' }}</td><td class="actions"><button class="icon-button" type="button" :title="item.enabled ? 'Disable subscription' : 'Enable subscription'" @click="action(() => api.setEnabled(item.id, !item.enabled))">{{ item.enabled ? 'Off' : 'On' }}</button><button class="icon-button" type="button" title="Run this subscription" @click="action(() => api.runSubscription(item.id))"><Play :size="15" /></button><button class="icon-button" type="button" title="Edit subscription" @click="edit(item)"><Pencil :size="15" /></button></td></tr><tr v-if="!subscriptions.length"><td colspan="6" class="empty-cell">No subscriptions.</td></tr></tbody></table></div></section>
  <section class="section-block" aria-labelledby="rss-form-heading"><h2 id="rss-form-heading">{{ editing ? 'Edit subscription' : 'New subscription' }}</h2><form class="organize-form" @submit.prevent="save"><div class="form-grid"><label class="form-field"><span>RSS URL</span><input v-model="form.url" type="url" required /></label><label class="form-field"><span>Target folder</span><input v-model="form.target_folder" required /></label><label class="form-field"><span>Filter regex</span><input v-model="form.filter_regex" /></label><label class="form-field"><span>Interval seconds</span><input v-model.number="form.interval_secs" type="number" min="30" required /></label><label class="form-field"><span>CloudDrive connection</span><select v-model.number="form.connection_id"><option :value="null">Choose a connection</option><option v-for="connection in connections" :key="connection.id" :value="connection.id">{{ connection.name }}</option></select></label></div><div class="form-actions"><button class="button secondary" type="button" @click="reset">Clear</button><button class="button primary">Save subscription</button></div></form></section>
</template>
