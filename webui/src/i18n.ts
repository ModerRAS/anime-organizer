import { ref } from 'vue'

export type Locale = 'en' | 'zh-CN'
export type MessageParams = Record<string, string | number>

const storageKey = 'anime-organizer.locale'

const zh: Record<string, string> = {
  '中文': '中文', 'English': 'English', 'Language': '语言', 'Loading...': '正在加载…', 'Saving...': '正在保存…',
  'Open navigation': '打开导航', 'Close navigation': '关闭导航', 'Primary navigation': '主导航',
  'Daemon online': 'Daemon 在线', 'Daemon unavailable': 'Daemon 不可用',
  'Dashboard': '仪表盘', 'Jobs': '任务', 'Organize': '整理', 'Scraper': '刮削器', 'Torrents': '种子', 'Aliases': '别名', 'RSS': 'RSS', 'CloudDrive': 'CloudDrive', 'About': '关于',
  '{count} job types enabled': '已启用 {count} 种任务',
  'System': '系统', 'Local daemon runtime and compiled feature set.': '本地 daemon 运行状态和已编译功能。', 'Daemon version': 'Daemon 版本', 'Uptime': '运行时间', 'Database': '数据库', 'Unavailable': '不可用', 'Runtime': '运行时', 'Capabilities': '功能', 'Capabilities are unavailable while the daemon is offline.': 'Daemon 离线时无法读取功能列表。', 'This page is unavailable in the current build: {feature}': '当前构建不支持此页面：{feature}',
  'Operations': '运行状态', 'Queue health and recent activity from the local daemon.': '本地 daemon 的队列健康状态和近期活动。', 'Refresh': '刷新', 'Queue summary': '队列摘要', 'Worker': 'Worker', 'unknown': '未知', 'No active job': '无活动任务', 'Queued': '排队中', 'Waiting to run': '等待执行', 'Running': '运行中', 'Single worker': '单 Worker', 'Completed': '已完成', '{count} failed': '{count} 个失败', 'Live state': '实时状态', 'Current job': '当前任务', 'Job ID': '任务 ID', 'Worker state': 'Worker 状态', 'The worker is idle. Jobs submitted through the API will appear here.': 'Worker 当前空闲，通过 API 提交的任务会显示在这里。', 'Needs attention': '需要处理', 'Recent failures': '近期失败', 'View all': '查看全部', 'Job': '任务', 'Type': '类型', 'State': '状态', 'Error': '错误', 'No error message': '无错误信息', 'No failed jobs in recent history.': '近期没有失败任务。',
  'Queue': '队列', 'Inspect accepted work and control jobs that have not started.': '查看已接收的任务并控制尚未开始的任务。', '{count} records': '{count} 条记录', 'Job filters': '任务筛选', 'All states': '全部状态', 'Succeeded': '成功', 'Failed': '失败', 'Canceled': '已取消', 'All types': '全部类型', 'Clear filters': '清除筛选', 'Daemon jobs': 'Daemon 任务', 'Origin': '来源', 'Created': '创建时间', 'Actions': '操作', 'Cancel queued job': '取消排队任务', 'Retry job': '重试任务', 'No jobs match these filters.': '没有符合筛选条件的任务。', 'Refreshing queue...': '正在刷新队列…', 'Load older': '加载更早任务', 'Loading older jobs...': '正在加载更早任务…', 'Cancel this queued job?': '确认取消这个排队任务？',
  'Back to jobs': '返回任务列表', 'Job detail': '任务详情', 'Cancel': '取消', 'Retry': '重试', 'Only queued jobs can be canceled': '只能取消排队中的任务', 'Loading job...': '正在加载任务…', 'Job not found.': '未找到任务。', 'Request': '请求', 'Parameters': '参数', 'Attempts': '尝试次数', 'Started': '开始时间', 'Finished': '完成时间', 'Duration': '耗时', 'Logs': '日志', 'Live execution log': '实时执行日志', 'No log entries yet.': '暂无日志。', 'Outcome': '结果', 'Result': '结果', 'Running jobs cannot be canceled because file operations must finish cleanly.': '运行中的文件操作必须完整结束，不能中途取消。', 'Artifacts': '产物', 'No final result yet.': '暂无最终结果。', '{size} bytes': '{size} 字节',
  'Manual job': '手动任务', 'Submit one complete organize request to the daemon queue.': '向 daemon 队列提交一份完整整理请求。', 'Defaults': '默认值', 'Presets': '预设', 'Saved preset': '已保存预设', 'Choose a preset': '选择预设', 'Load': '加载', 'Delete selected preset': '删除所选预设', 'Delete this preset?': '确认删除这个预设？', 'New preset name': '新预设名称', 'Optional saved name': '可选保存名称', 'Save preset': '保存预设', 'Preset name is required.': '必须填写预设名称。', 'Required': '必填', 'Paths and operation': '路径和操作', 'Source': '来源', 'Target': '目标', 'Mode': '模式', 'Hard link': '硬链接', 'Copy': '复制', 'Move': '移动', 'Link failure fallback': '链接失败回退', 'No fallback': '不回退', 'Dry run': '试运行', 'Optional': '可选', 'Advanced options': '高级选项', 'Included extensions': '包含扩展名', 'Filename parser': '文件名解析器', 'Rules': '规则', 'Auto': '自动', 'TMDB API key': 'TMDB API Key', 'Optional key': '可选 Key', 'Alias file': '别名文件', 'Optional JSON path': '可选 JSON 路径', 'Bangumi cache': 'Bangumi 缓存', 'Optional cache path': '可选缓存路径', 'Metadata source': '元数据来源', 'Optional local subject.jsonlines path': '可选本地 subject.jsonlines 路径', 'Verbose logging': '详细日志', 'Scrape metadata': '刮削元数据', 'Skip images': '跳过图片', 'Skip episode metadata': '跳过剧集元数据', 'Overwrite existing metadata': '覆盖现有元数据', 'Season mode': '分季模式', 'Update library index': '更新媒体库索引', 'Build MLIP library': '构建 MLIP 媒体库', 'Rebuild library index': '重建媒体库索引', 'Probe runtime with ffprobe': '使用 ffprobe 探测时长', 'Confirmation required': '需要确认', 'I understand this request can change files or rebuild the library index.': '我了解此请求会更改文件或重建媒体库索引。', 'Submitting...': '正在提交…', 'Submit organize job': '提交整理任务', 'Required fields are marked *': '带 * 的字段为必填项', 'Source is required.': '必须填写来源路径。', 'Target is required.': '必须填写目标路径。', 'Enable library index or MLIP before rebuilding.': '重建前请启用媒体库索引或 MLIP。', 'Confirm this move or index rebuild before submitting.': '提交移动或索引重建前请先确认。',
  'Data workflows': '数据工作流', 'Run bounded scrape and alias matching jobs through the daemon queue.': '通过 daemon 队列运行有边界的刮削和别名匹配任务。', 'Refresh scraper jobs': '刷新刮削任务', 'Scraper actions': '刮削操作', 'Sources': '数据源', 'Scrape recent anime': '刮削近期动画', 'Days': '天数', 'Output mode': '输出模式', 'Pretty': '易读格式', '(optional)': '（可选）', 'Results stay in the job record or an artifact.': '结果保存在任务记录或产物中。', 'Queue scrape': '加入刮削队列', 'Matching': '匹配', 'Match aliases': '匹配别名', 'Scrape JSON path': '刮削 JSON 路径', 'Result mode': '结果模式', 'JSON + GitHub artifact': 'JSON + GitHub 产物', 'The worker reads the local input when the job starts.': 'Worker 在任务开始时读取本地输入。', 'Queue matching': '加入匹配队列', 'History': '历史', 'Scraper jobs': '刮削任务', 'Downloads': '下载', 'Inline result': '内联结果', 'No scraper jobs yet.': '还没有刮削任务。', 'Refreshing scraper history...': '正在刷新刮削历史…',
  'Torrent sources': '种子来源', 'DMHY and Nyaa': 'DMHY 和 Nyaa', 'Queue bounded source scrapes and review unique torrent filenames.': '提交有边界的来源刮削并查看去重后的种子文件名。', 'Refresh torrent jobs': '刷新种子任务', 'New scrape': '新建刮削', 'Source parameters': '来源参数', 'Pages': '页数', 'Nyaa search query': 'Nyaa 搜索词', 'Output path': '输出路径', 'Pages are clamped to 1-2000. Results include a text artifact and bounded preview.': '页数限制为 1–2000，结果包含文本产物和有边界的预览。', 'Torrent scrape jobs': '种子刮削任务', 'Unique titles': '唯一标题', 'Preview': '预览', 'Artifact': '产物', 'Preview truncated': '预览已截断', 'No result yet': '暂无结果', 'No torrent scrape jobs yet.': '还没有种子刮削任务。', 'Refreshing torrent history...': '正在刷新种子历史…',
  'Alias maintenance': '别名维护', 'Build, extract, review, and maintain the Bangumi alias database through typed queued jobs.': '通过 typed 队列任务构建、提取、审查和维护 Bangumi 别名数据库。', 'Refresh alias jobs': '刷新别名任务', 'Alias maintenance actions': '别名维护操作', 'Build Bangumi database': '构建 Bangumi 数据库', 'Include relations': '包含关联', 'Verbose worker output': '详细 Worker 输出', 'Downloads the current Bangumi archive.': '下载当前 Bangumi Archive。', 'Queue build': '加入构建队列', 'Extraction': '提取', 'Extract aliases': '提取别名', 'Dump path': 'Dump 路径', 'optional when downloading': '下载时可不填', 'Download latest dump': '下载最新 Dump', 'The JSON result is saved as a job artifact.': 'JSON 结果保存为任务产物。', 'Queue extraction': '加入提取队列', 'Mutation': '写入操作', 'Merge aliases': '合并别名', 'Alias JSON path': '别名 JSON 路径', 'Target database': '目标数据库', 'Writes only to the selected target database.': '仅写入所选目标数据库。', 'Queue merge': '加入合并队列', 'Apply matches': '应用匹配', 'Proposal JSON path': '提案 JSON 路径', 'Only confident proposals are applied.': '仅应用高置信度提案。', 'Queue apply': '加入应用队列', 'Review': '审查', 'Create GitHub issues': '创建 GitHub Issues', 'Uncertain proposal JSON path': '不确定提案 JSON 路径', 'Repository': '仓库', 'Each proposal receives an individual result.': '每个提案都会获得独立结果。', 'Queue issue creation': '加入 Issue 创建队列', 'I confirm database mutations and external issue creation': '我确认数据库写入和外部 Issue 创建操作', 'Maintenance jobs': '维护任务', 'No alias maintenance jobs yet.': '还没有别名维护任务。', 'Refreshing alias history...': '正在刷新别名历史…',
  'CloudDrive feed intake': 'CloudDrive Feed 接入', 'RSS subscriptions': 'RSS 订阅', 'Manage polling intervals, filters, destinations, and download history.': '管理轮询间隔、筛选器、目标目录和下载历史。', 'Refresh subscriptions': '刷新订阅', 'New subscription': '新建订阅', 'Saved subscriptions': '已保存订阅', 'Run all': '全部运行', 'Run all subscriptions now?': '确认立即运行全部订阅？', 'URL': 'URL', 'Interval': '间隔', 'Last checked': '上次检查', 'No filter': '无筛选器', 'Enabled': '已启用', 'Disabled': '已禁用', 'Never': '从未', 'Disable subscription': '禁用订阅', 'Enable subscription': '启用订阅', 'Run this subscription': '运行此订阅', 'Run this subscription now?': '确认立即运行此订阅？', 'Edit subscription': '编辑订阅', 'Delete subscription': '删除订阅', 'Delete this subscription?': '确认删除此订阅？', 'No subscriptions.': '没有订阅。', 'RSS URL': 'RSS URL', 'Target folder': '目标目录', 'Filter regex': '筛选正则', 'Interval seconds': '间隔秒数', 'CloudDrive connection': 'CloudDrive 连接', 'Choose a connection': '选择连接', 'Clear': '清除', 'Save subscription': '保存订阅', 'A CloudDrive connection is required.': '必须选择 CloudDrive 连接。', 'Subscription queued as job #{id}': '订阅已加入任务队列 #{id}',
  'Subscription saved.': '订阅已保存。', 'Subscription updated.': '订阅已更新。', 'Subscription deleted.': '订阅已删除。', 'Connection required': '需要连接', '{seconds}s': '{seconds} 秒', '{minutes}m': '{minutes} 分钟',
  'Back to RSS': '返回 RSS', 'Subscription details': '订阅详情', 'Subscription not found.': '未找到订阅。', 'Loading subscription...': '正在加载订阅…', 'Processed items': '已处理条目', 'Title': '标题', 'Hash': 'Hash', 'Processed': '处理时间', 'Untitled': '无标题', 'No processed items.': '没有已处理条目。', 'Download tasks': '下载任务', 'Cloud': 'Cloud', 'Status': '状态', 'Added': '添加时间', 'No download tasks.': '没有下载任务。',
  'Saved credentials': '已保存凭据', 'CloudDrive connections': 'CloudDrive 连接', 'Credentials are stored by the local daemon and shown here only as presence flags.': '凭据由本地 daemon 保存，此处只显示是否已设置。', 'A token or username/password login is required.': '必须提供 Token 或用户名/密码登录信息。', 'Username and password must be provided together.': '用户名和密码必须同时提供。', 'Name': '名称', 'Endpoint': '端点', 'Credentials': '凭据', 'Token set': '已设置 Token', 'Login set': '已设置登录信息', 'No credentials': '无凭据', 'Browse folder': '浏览目录', 'Test connection': '测试连接', 'Delete connection': '删除连接', 'Delete this connection?': '确认删除此连接？', 'Connection test succeeded.': '连接测试成功。', 'Connection saved.': '连接已保存。', 'Connection deleted.': '连接已删除。', 'No connections.': '没有连接。', 'Add connection': '添加连接', 'Token': 'Token', 'Username': '用户名', 'Password': '密码', 'Save connection': '保存连接', 'Folder browser / {name}': '目录浏览 / {name}', 'Root': '根目录', 'Offline URL': '离线 URL', 'Queue this offline URL?': '确认提交此离线 URL？', 'Queued job {id}': '已加入任务 #{id}', 'Queue URL': '提交 URL', 'Path': '路径', 'Size': '大小', 'Folder': '目录', 'File': '文件', 'Folder is empty.': '目录为空。',
  'AniFileBERT': 'AniFileBERT', 'DMHY': 'DMHY', 'Nyaa': 'Nyaa', 'JSON': 'JSON', 'mp4,mkv,avi': 'mp4,mkv,avi', 'metadata': '元数据', 'anifilebert': 'AniFileBERT', 'clouddrive': 'CloudDrive', 'scraper': '刮削器', 'torrent-scraper': '种子刮削器',
  'idle': '空闲', 'validating': '校验中', 'running': '运行中', 'stopping': '停止中', 'error': '错误', 'queued': '排队中', 'succeeded': '成功', 'failed': '失败', 'canceled': '已取消',
  'manual': '手动', 'scheduled': '计划任务', 'qbittorrent': 'qBittorrent',
  'organize': '整理', 'scrape': '刮削', 'match_aliases': '匹配别名', 'build_bangumi_db': '构建 Bangumi 数据库', 'extract_aliases': '提取别名', 'merge_aliases': '合并别名', 'apply_matches': '应用匹配', 'create_alias_issues': '创建别名 Issues', 'torrent_scrape': '种子刮削', 'rss_poll': 'RSS 轮询', 'rss_poll_all': '全部 RSS 轮询', 'cloud_add_offline': 'CloudDrive 离线下载',
}

function initialLocale(): Locale {
  try {
    const saved = window.localStorage.getItem(storageKey)
    if (saved === 'en' || saved === 'zh-CN') return saved
  } catch { /* Storage may be unavailable. */ }
  return typeof navigator !== 'undefined' && navigator.language.toLowerCase().startsWith('zh') ? 'zh-CN' : 'en'
}

export const locale = ref<Locale>(initialLocale())

export function setLocale(value: Locale) {
  locale.value = value
  if (typeof document !== 'undefined') document.documentElement.lang = value
  try { window.localStorage.setItem(storageKey, value) } catch { /* Storage may be unavailable. */ }
}

export function t(key: string, params: MessageParams = {}): string {
  const template = locale.value === 'zh-CN' ? (zh[key] ?? key) : key
  return template.replace(/\{(\w+)\}/g, (_, name: string) => String(params[name] ?? `{${name}}`))
}

export function valueLabel(value: string): string {
  const translated = t(value)
  return translated === value ? value.replaceAll('_', ' ') : translated
}

function parseDateTime(value: string | null | undefined): Date | null {
  if (!value) return null
  let normalized = value
  if (/^\d+$/.test(value)) normalized = String(Number(value) * 1000)
  else if (/^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$/.test(value)) normalized = `${value.replace(' ', 'T')}Z`
  const date = /^\d+$/.test(normalized) ? new Date(Number(normalized)) : new Date(normalized)
  return Number.isNaN(date.getTime()) ? null : date
}

export function formatDateTime(value: string | null | undefined): string {
  return parseDateTime(value)?.toLocaleString(locale.value === 'zh-CN' ? 'zh-CN' : 'en-US') ?? '-'
}

export function formatDuration(started: string | null, finished: string | null): string {
  const start = parseDateTime(started)
  if (!start) return '-'
  const end = parseDateTime(finished) ?? new Date()
  const total = Math.max(0, Math.floor((end.getTime() - start.getTime()) / 1000))
  const hours = Math.floor(total / 3600)
  const minutes = Math.floor((total % 3600) / 60)
  const seconds = total % 60
  return hours > 0
    ? `${hours}:${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`
    : `${minutes}:${String(seconds).padStart(2, '0')}`
}

setLocale(locale.value)
