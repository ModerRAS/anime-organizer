<script setup lang="ts">
import { computed, ref } from 'vue'
import { RouterLink, RouterView } from 'vue-router'
import { Activity, BriefcaseBusiness, ClipboardList, Cloud, FileSearch, Menu, Monitor, Radio, Tags, X } from 'lucide-vue-next'
import { locale, setLocale, t } from './i18n'
import { useStatus } from './stores/status'

const navOpen = ref(false)
const { state } = useStatus()
const online = computed(() => state.health?.status === 'ok')
const clouddriveAvailable = computed(() => state.capabilities?.features.includes('clouddrive') === true)
const scraperAvailable = computed(() => state.capabilities?.job_types.includes('scrape') === true && state.capabilities?.job_types.includes('match_aliases') === true)
const torrentAvailable = computed(() => state.capabilities?.job_types.includes('torrent_scrape') === true)
const aliasesAvailable = computed(() => state.capabilities?.job_types.includes('build_bangumi_db') === true)

function closeNav() {
  navOpen.value = false
}
</script>

<template>
  <div class="app-shell">
    <header class="topbar">
      <button class="icon-button menu-button" type="button" :aria-label="t(navOpen ? 'Close navigation' : 'Open navigation')" :title="t(navOpen ? 'Close navigation' : 'Open navigation')" :aria-expanded="navOpen" aria-controls="primary-navigation" @click="navOpen = !navOpen">
        <X v-if="navOpen" :size="19" aria-hidden="true" />
        <Menu v-else :size="19" aria-hidden="true" />
      </button>
      <RouterLink class="brand" to="/" @click="closeNav">
        <span class="brand-mark"><Activity :size="17" aria-hidden="true" /></span>
        <span>anime-organizer</span>
      </RouterLink>
      <div class="topbar-actions">
        <div class="locale-switch" role="group" :aria-label="t('Language')">
          <button type="button" :class="{ active: locale === 'zh-CN' }" :aria-pressed="locale === 'zh-CN'" @click="setLocale('zh-CN')">中文</button>
          <button type="button" :class="{ active: locale === 'en' }" :aria-pressed="locale === 'en'" @click="setLocale('en')">EN</button>
        </div>
        <div class="daemon-indicator" :class="{ online }" role="status">
          <span class="status-dot" aria-hidden="true"></span>
          <span>{{ t(online ? 'Daemon online' : 'Daemon unavailable') }}</span>
          <span v-if="state.health" class="version">v{{ state.health.version }}</span>
        </div>
      </div>
    </header>

    <div class="shell-body">
      <aside id="primary-navigation" class="sidebar" :class="{ open: navOpen }" :aria-label="t('Primary navigation')">
        <nav class="nav-list">
          <RouterLink to="/" @click="closeNav"><Monitor :size="17" aria-hidden="true" />{{ t('Dashboard') }}</RouterLink>
          <RouterLink to="/jobs" @click="closeNav"><BriefcaseBusiness :size="17" aria-hidden="true" />{{ t('Jobs') }}</RouterLink>
          <RouterLink to="/organize" @click="closeNav"><ClipboardList :size="17" aria-hidden="true" />{{ t('Organize') }}</RouterLink>
          <RouterLink v-if="scraperAvailable" to="/scraper" @click="closeNav"><FileSearch :size="17" aria-hidden="true" />{{ t('Scraper') }}</RouterLink>
          <RouterLink v-if="torrentAvailable" to="/torrent" @click="closeNav"><FileSearch :size="17" aria-hidden="true" />{{ t('Torrents') }}</RouterLink>
          <RouterLink v-if="aliasesAvailable" to="/aliases" @click="closeNav"><Tags :size="17" aria-hidden="true" />{{ t('Aliases') }}</RouterLink>
          <RouterLink v-if="clouddriveAvailable" to="/rss" @click="closeNav"><Radio :size="17" aria-hidden="true" />{{ t('RSS') }}</RouterLink>
          <RouterLink v-if="clouddriveAvailable" to="/cloud" @click="closeNav"><Cloud :size="17" aria-hidden="true" />{{ t('CloudDrive') }}</RouterLink>
        </nav>
        <div class="sidebar-footer">
          <RouterLink to="/about" @click="closeNav">{{ t('About') }}</RouterLink>
          <span v-if="state.capabilities" class="capability-count">{{ t('{count} job types enabled', { count: state.capabilities.job_types.length }) }}</span>
        </div>
      </aside>
      <button v-if="navOpen" class="nav-scrim" type="button" :aria-label="t('Close navigation')" @click="closeNav"></button>
      <main class="main-content" @click="closeNav">
        <RouterView />
      </main>
    </div>
  </div>
</template>
