export type Breadcrumb = { label: string; path: string }

export function breadcrumbEntries(value: string): Breadcrumb[] {
  let current = ''
  return value
    .split('/')
    .filter(Boolean)
    .map((label) => {
      current += `/${label}`
      return { label, path: current }
    })
}
