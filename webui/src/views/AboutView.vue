<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { t, valueLabel } from '../i18n'
import { useStatus } from '../stores/status'

const route = useRoute()
const { state } = useStatus()
const unavailableFeature = computed(() => {
  const value = route.query.unavailable
  return Array.isArray(value) ? value[0] ?? '' : value ?? ''
})
</script>

<template>
  <p v-if="route.query.unavailable !== undefined" class="alert error" role="alert">{{ t('This page is unavailable in the current build: {feature}', { feature: unavailableFeature }) }}</p>
  <div class="page-header"><div><p class="eyebrow">{{ t('System') }}</p><h1>{{ t('About') }}</h1><p class="page-subtitle">{{ t('Local daemon runtime and compiled feature set.') }}</p></div></div>
  <section class="section-block about-grid"><div><span class="label">{{ t('Daemon version') }}</span><strong>{{ state.health?.version ?? t('Unavailable') }}</strong></div><div><span class="label">{{ t('Uptime') }}</span><strong>{{ state.status ? t('{seconds}s', { seconds: state.status.uptime_seconds }) : t('Unavailable') }}</strong></div><div><span class="label">{{ t('Database') }}</span><code>{{ state.status?.database_path ?? t('Unavailable') }}</code></div></section>
  <section class="section-block" aria-labelledby="capabilities-heading"><div class="section-heading"><div><p class="eyebrow">{{ t('Runtime') }}</p><h2 id="capabilities-heading">{{ t('Capabilities') }}</h2></div></div><div v-if="state.capabilities" class="capability-list"><span v-for="feature in state.capabilities.features" :key="feature" class="capability">{{ valueLabel(feature) }}</span></div><p v-else class="empty-state">{{ t('Capabilities are unavailable while the daemon is offline.') }}</p></section>
</template>
