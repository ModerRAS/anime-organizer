<script setup lang="ts">
import { useStatus } from '../stores/status'
const { state } = useStatus()
</script>

<template>
  <div class="page-header"><div><p class="eyebrow">System</p><h1>About</h1><p class="page-subtitle">Local daemon runtime and compiled feature set.</p></div></div>
  <section class="section-block about-grid"><div><span class="label">Daemon version</span><strong>{{ state.health?.version ?? 'Unavailable' }}</strong></div><div><span class="label">Uptime</span><strong>{{ state.status ? `${state.status.uptime_seconds}s` : 'Unavailable' }}</strong></div><div><span class="label">Database</span><code>{{ state.status?.database_path ?? 'Unavailable' }}</code></div></section>
  <section class="section-block" aria-labelledby="capabilities-heading"><div class="section-heading"><div><p class="eyebrow">Runtime</p><h2 id="capabilities-heading">Capabilities</h2></div></div><div v-if="state.capabilities" class="capability-list"><span v-for="feature in state.capabilities.features" :key="feature" class="capability">{{ feature }}</span></div><p v-else class="empty-state">Capabilities are unavailable while the daemon is offline.</p></section>
</template>
