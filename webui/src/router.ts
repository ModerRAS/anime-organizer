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

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', component: DashboardView },
    { path: '/jobs', component: JobsView },
    { path: '/jobs/:id', component: JobDetailView },
    { path: '/organize', component: OrganizeView },
    { path: '/scraper', component: ScraperView },
    { path: '/torrent', component: TorrentView },
    { path: '/aliases', component: AliasesView },
    { path: '/rss', component: RssView },
    { path: '/rss/:id', component: RssDetailView },
    { path: '/cloud', component: CloudView },
    { path: '/about', component: AboutView },
  ],
})
