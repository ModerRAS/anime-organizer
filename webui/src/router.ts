import { createRouter, createWebHistory } from 'vue-router'
import DashboardView from './views/DashboardView.vue'
import JobsView from './views/JobsView.vue'
import JobDetailView from './views/JobDetailView.vue'
import AboutView from './views/AboutView.vue'
import OrganizeView from './views/OrganizeView.vue'
import ScraperView from './views/ScraperView.vue'
import TorrentView from './views/TorrentView.vue'
import AliasesView from './views/AliasesView.vue'
import RssView from './views/RssView.vue'
import RssDetailView from './views/RssDetailView.vue'
import CloudView from './views/CloudView.vue'
import { api, type Capabilities } from './api'

type CapabilityRequirement = { field: keyof Pick<Capabilities, 'features' | 'job_types' | 'resources'>; values: string[] }

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', component: DashboardView },
    { path: '/jobs', component: JobsView },
    { path: '/jobs/:id', component: JobDetailView },
    { path: '/organize', component: OrganizeView },
    { path: '/scraper', component: ScraperView, meta: { capability: { field: 'job_types', values: ['scrape', 'match_aliases'] } } },
    { path: '/torrent', component: TorrentView, meta: { capability: { field: 'job_types', values: ['torrent_scrape'] } } },
    { path: '/aliases', component: AliasesView, meta: { capability: { field: 'job_types', values: ['build_bangumi_db'] } } },
    { path: '/rss', component: RssView, meta: { capability: { field: 'resources', values: ['rss_subscriptions'] } } },
    { path: '/rss/:id', component: RssDetailView, meta: { capability: { field: 'resources', values: ['rss_subscriptions'] } } },
    { path: '/cloud', component: CloudView, meta: { capability: { field: 'features', values: ['clouddrive'] } } },
    { path: '/about', component: AboutView },
    { path: '/:pathMatch(.*)*', redirect: '/' },
  ],
})

router.beforeEach(async (to) => {
  const requirement = to.meta.capability as CapabilityRequirement | undefined
  if (!requirement) return true
  try {
    const available = (await api.capabilities())[requirement.field]
    const missing = requirement.values.find(value => !available.includes(value))
    return missing ? { path: '/about', query: { unavailable: missing } } : true
  } catch {
    return true
  }
})
