---
name: tanstack-query
description: "TanStack Query (React Query) v5 best practices for fetching, caching, synchronizing and updating server state in React applications. Use when writing, reviewing, or refactoring code that involves: (1) Data fetching with useQuery, useSuspenseQuery, useInfiniteQuery, (2) Mutations with useMutation and cache invalidation, (3) Query key design and query function patterns, (4) Optimistic updates and cache manipulation, (5) Pagination and infinite scroll implementations, (6) Prefetching and request waterfall optimization, (7) Server-side rendering (SSR) with Next.js pages/app router or Remix, (8) Suspense integration and streaming with Server Components, (9) TanStack Query TypeScript patterns (queryOptions, type inference, global type registration), (10) Testing components that use React Query hooks, (11) Setting up QueryClient, QueryClientProvider, staleTime, gcTime, retry configuration, (12) Any @tanstack/react-query imports or configuration."
---

# TanStack Query (React Query) v5

## Quick Setup

```tsx
import { QueryClient, QueryClientProvider, useQuery } from '@tanstack/react-query'

const queryClient = new QueryClient()

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <MyComponent />
    </QueryClientProvider>
  )
}

function MyComponent() {
  const { data, isPending, error } = useQuery({
    queryKey: ['todos'],
    queryFn: () => fetch('/api/todos').then(res => res.json()),
  })
}
```

## Key API Patterns

### useQuery
```tsx
const { data, isPending, isError, error, isFetching } = useQuery({
  queryKey: ['todos', { status, page }],
  queryFn: () => fetchTodos({ status, page }),
  staleTime: 5 * 60 * 1000,
  enabled: !!userId,
})
```

### useMutation + Invalidation
```tsx
const queryClient = useQueryClient()
const mutation = useMutation({
  mutationFn: (newTodo) => axios.post('/todos', newTodo),
  onSuccess: () => queryClient.invalidateQueries({ queryKey: ['todos'] }),
})
mutation.mutate({ title: 'New Todo' })
```

### queryOptions (type-safe shared options)
```tsx
const todosOptions = (filters: Filters) => queryOptions({
  queryKey: ['todos', filters],
  queryFn: () => fetchTodos(filters),
  staleTime: 5000,
})

useQuery(todosOptions(filters))
queryClient.prefetchQuery(todosOptions(filters))
```

### Infinite Queries
```tsx
const { data, fetchNextPage, hasNextPage } = useInfiniteQuery({
  queryKey: ['projects'],
  queryFn: ({ pageParam }) => fetchProjects(pageParam),
  initialPageParam: 0,
  getNextPageParam: (lastPage) => lastPage.nextCursor,
})
```

### Suspense
```tsx
const { data } = useSuspenseQuery({ queryKey: ['todos'], queryFn: fetchTodos })
// data is always defined - errors/loading handled by Suspense/ErrorBoundary
```

## Critical Defaults to Remember

- `staleTime: 0` - Data considered stale immediately (refetches on mount/focus/reconnect)
- `gcTime: 5 * 60 * 1000` - Unused queries garbage collected after 5 min
- `retry: 3` on client, `0` on server - Exponential backoff
- `refetchOnWindowFocus: true` - Refetches stale queries on tab focus
- `structuralSharing: true` - Preserves referential identity for unchanged data
- Error type defaults to `Error` (configurable via Register interface)

## Reference Documentation

Consult these files for detailed patterns and examples:

- **Core concepts** (queries, mutations, keys, functions, invalidation, filters): See [references/core-concepts.md](references/core-concepts.md)
- **Advanced patterns** (pagination, infinite queries, optimistic updates, prefetching, dependent queries, cancellation, render optimizations): See [references/advanced-patterns.md](references/advanced-patterns.md)
- **SSR, Suspense & Server Components** (Next.js pages/app router, Remix, hydration, streaming, testing): See [references/ssr-and-suspense.md](references/ssr-and-suspense.md)
- **TypeScript** (type inference, global types, queryOptions, skipToken, GraphQL): See [references/typescript.md](references/typescript.md)

## Common Pitfalls

- **Don't create QueryClient at module level** in SSR - leaks data between requests. Create inside component state or use `getQueryClient()` pattern.
- **`fetch` doesn't throw on HTTP errors** - must check `response.ok` and throw manually in `queryFn`.
- **Query keys are arrays** - `['todos']` not `'todos'`. Object key order doesn't matter, array order does.
- **`enabled: false`** prevents all automatic fetching. Use `skipToken` for type-safe conditional disabling.
- **Mutations don't retry** by default (unlike queries). Set `retry` explicitly if needed.
- **`setQueryData` requires immutability** - never mutate cached data directly, always return new objects.
- **Suspense queries run serially** in same component - use `useSuspenseQueries` for parallel execution.
- **`placeholderData: keepPreviousData`** replaces the removed `keepPreviousData` option from v4.
