import { describe, expect, it } from 'vitest'
import { breadcrumbEntries } from './cloud'

describe('breadcrumbEntries', () => {
  it('keeps the complete prefix for nested folders', () => {
    expect(breadcrumbEntries('/foo/bar/baz')).toEqual([
      { label: 'foo', path: '/foo' },
      { label: 'bar', path: '/foo/bar' },
      { label: 'baz', path: '/foo/bar/baz' },
    ])
  })
})
