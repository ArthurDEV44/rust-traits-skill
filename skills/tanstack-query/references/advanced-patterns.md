# TanStack Query Advanced Patterns

## Table of Contents
- [Parallel Queries](#parallel-queries)
- [Dependent Queries](#dependent-queries)
- [Paginated Queries](#paginated-queries)
- [Infinite Queries](#infinite-queries)
- [Optimistic Updates](#optimistic-updates)
- [Query Cancellation](#query-cancellation)
- [Disabling Queries](#disabling-queries)
- [Query Retries](#query-retries)
- [Initial Query Data](#initial-query-data)
- [Placeholder Query Data](#placeholder-query-data)
- [Default Query Function](#default-query-function)
- [Render Optimizations](#render-optimizations)
- [Window Focus Refetching](#window-focus-refetching)
- [Network Mode](#network-mode)
- [Background Fetching Indicators](#background-fetching-indicators)
- [Prefetching](#prefetching)

## Parallel Queries

Use multiple `useQuery` hooks side-by-side:
```tsx
const usersQuery = useQuery({ queryKey: ['users'], queryFn: fetchUsers })
const teamsQuery = useQuery({ queryKey: ['teams'], queryFn: fetchTeams })
```

For dynamic number of parallel queries, use `useQueries`:
```tsx
const userQueries = useQueries({
  queries: users.map((user) => ({
    queryKey: ['user', user.id],
    queryFn: () => fetchUserById(user.id),
  })),
})
```

With suspense, use `useSuspenseQueries` instead (individual `useSuspenseQuery` calls run serially).

## Dependent Queries

Use `enabled` to create serial/dependent queries:

```tsx
const { data: user } = useQuery({
  queryKey: ['user', email],
  queryFn: getUserByEmail,
})

const { data: projects } = useQuery({
  queryKey: ['projects', user?.id],
  queryFn: getProjectsByUser,
  enabled: !!user?.id,
})
```

Dependent `useQueries`:
```tsx
const { data: userIds } = useQuery({
  queryKey: ['users'],
  queryFn: getUsersData,
  select: (users) => users.map((user) => user.id),
})

const usersMessages = useQueries({
  queries: userIds
    ? userIds.map((id) => ({
        queryKey: ['messages', id],
        queryFn: () => getMessagesByUsers(id),
      }))
    : [],
})
```

**Performance note**: Dependent queries create request waterfalls. Prefer restructuring APIs (e.g., `getProjectsByUserEmail`) to flatten waterfalls when possible.

## Paginated Queries

Include page in query key. Use `placeholderData` with `keepPreviousData` for smooth transitions:

```tsx
import { keepPreviousData, useQuery } from '@tanstack/react-query'

const { data, isPlaceholderData } = useQuery({
  queryKey: ['projects', page],
  queryFn: () => fetchProjects(page),
  placeholderData: keepPreviousData,
})

// Disable next page button while showing placeholder
<button
  disabled={isPlaceholderData || !data?.hasMore}
  onClick={() => setPage(old => old + 1)}
>
  Next Page
</button>
```

## Infinite Queries

```tsx
const {
  data,
  fetchNextPage,
  hasNextPage,
  isFetchingNextPage,
} = useInfiniteQuery({
  queryKey: ['projects'],
  queryFn: ({ pageParam }) => fetchProjects(pageParam),
  initialPageParam: 0,
  getNextPageParam: (lastPage, pages) => lastPage.nextCursor,
})

// data.pages = array of fetched pages
// data.pageParams = array of page params
```

**Bi-directional**: Add `getPreviousPageParam`, use `fetchPreviousPage`, `hasPreviousPage`.

**Reversed order**: Use `select`:
```tsx
select: (data) => ({
  pages: [...data.pages].reverse(),
  pageParams: [...data.pageParams].reverse(),
})
```

**Limit pages in memory** with `maxPages`:
```tsx
useInfiniteQuery({
  queryKey: ['projects'],
  queryFn: fetchProjects,
  initialPageParam: 0,
  getNextPageParam: (lastPage, pages) => lastPage.nextCursor,
  getPreviousPageParam: (firstPage, pages) => firstPage.prevCursor,
  maxPages: 3,
})
```

**Manual updates**:
```tsx
// Remove first page
queryClient.setQueryData(['projects'], (data) => ({
  pages: data.pages.slice(1),
  pageParams: data.pageParams.slice(1),
}))
```

**No cursor from API?** Use pageParam as cursor:
```tsx
getNextPageParam: (lastPage, allPages, lastPageParam) => {
  if (lastPage.length === 0) return undefined
  return lastPageParam + 1
},
```

## Optimistic Updates

### Via the UI (simpler, recommended for single location)

```tsx
const { isPending, variables, mutate, isError } = useMutation({
  mutationFn: (newTodo: string) => axios.post('/api/data', { text: newTodo }),
  onSettled: () => queryClient.invalidateQueries({ queryKey: ['todos'] }),
})

// Render optimistic item:
{isPending && <li style={{ opacity: 0.5 }}>{variables}</li>}
{isError && (
  <li style={{ color: 'red' }}>
    {variables}
    <button onClick={() => mutate(variables)}>Retry</button>
  </li>
)}
```

Access mutation variables from other components via `useMutationState`:
```tsx
const variables = useMutationState<string>({
  filters: { mutationKey: ['addTodo'], status: 'pending' },
  select: (mutation) => mutation.state.variables,
})
```

### Via the cache (automatic multi-location updates)

```tsx
useMutation({
  mutationFn: updateTodo,
  onMutate: async (newTodo, context) => {
    await context.client.cancelQueries({ queryKey: ['todos'] })
    const previousTodos = context.client.getQueryData(['todos'])
    context.client.setQueryData(['todos'], (old) => [...old, newTodo])
    return { previousTodos }
  },
  onError: (err, newTodo, onMutateResult, context) => {
    context.client.setQueryData(['todos'], onMutateResult.previousTodos)
  },
  onSettled: (data, error, variables, onMutateResult, context) =>
    context.client.invalidateQueries({ queryKey: ['todos'] }),
})
```

## Query Cancellation

TanStack Query provides an `AbortSignal` via `QueryFunctionContext`:

```tsx
useQuery({
  queryKey: ['todos'],
  queryFn: async ({ signal }) => {
    const response = await fetch('/todos', { signal })
    return response.json()
  },
})
```

Works with axios:
```tsx
queryFn: ({ signal }) => axios.get('/todos', { signal })
```

Manual cancellation:
```tsx
queryClient.cancelQueries({ queryKey: ['todos'] })
```

Cancel options: `{ silent?: boolean, revert?: boolean }`

**Limitation**: Cancellation does not work with Suspense hooks.

## Disabling Queries

```tsx
// With enabled (allows refetch)
useQuery({
  queryKey: ['todos'],
  queryFn: fetchTodoList,
  enabled: false,
})

// With skipToken (type-safe, refetch won't work)
import { skipToken } from '@tanstack/react-query'

useQuery({
  queryKey: ['todos', filter],
  queryFn: filter ? () => fetchTodos(filter) : skipToken,
})
```

**Lazy queries** with conditional `enabled`:
```tsx
const { data } = useQuery({
  queryKey: ['todos', filter],
  queryFn: () => fetchTodos(filter),
  enabled: !!filter,
})
```

`isLoading` = `isPending && isFetching` (true only when actually fetching for the first time).

## Query Retries

```tsx
const result = useQuery({
  queryKey: ['todos', 1],
  queryFn: fetchTodoListPage,
  retry: 10, // default: 3 (0 on server)
  retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
})
```

Options: `false` (disable), number, `true` (infinite), `(failureCount, error) => boolean`.

## Initial Query Data

Persist to cache with `initialData`:
```tsx
const result = useQuery({
  queryKey: ['todos'],
  queryFn: () => fetch('/todos'),
  initialData: initialTodos,
  // Optional: when was this data last fresh?
  initialDataUpdatedAt: initialTodosUpdatedTimestamp,
})
```

From another query's cache:
```tsx
const result = useQuery({
  queryKey: ['todo', todoId],
  queryFn: () => fetch('/todos'),
  initialData: () =>
    queryClient.getQueryData(['todos'])?.find((d) => d.id === todoId),
  initialDataUpdatedAt: () =>
    queryClient.getQueryState(['todos'])?.dataUpdatedAt,
})
```

## Placeholder Query Data

Not persisted to cache. Query starts in `success` state with `isPlaceholderData: true`:

```tsx
const result = useQuery({
  queryKey: ['todos'],
  queryFn: () => fetch('/todos'),
  placeholderData: placeholderTodos,
})

// Function form (access previous data for transitions):
const result = useQuery({
  queryKey: ['todos', id],
  queryFn: () => fetch(`/todos/${id}`),
  placeholderData: (previousData, previousQuery) => previousData,
})
```

## Default Query Function

```tsx
const defaultQueryFn = async ({ queryKey }) => {
  const { data } = await axios.get(`https://api.example.com${queryKey[0]}`)
  return data
}

const queryClient = new QueryClient({
  defaultOptions: { queries: { queryFn: defaultQueryFn } },
})

// Then just pass keys:
useQuery({ queryKey: ['/posts'] })
```

## Render Optimizations

- **Structural sharing**: Unchanged data keeps same reference.
- **Tracked properties**: Only re-renders when accessed properties change (via Proxy). Don't use rest destructuring (`const { data, ...rest } = useQuery(...)` disables tracking).
- **`select`**: Subscribe to subset of data:

```tsx
const { data } = useQuery({
  queryKey: ['todos'],
  queryFn: fetchTodos,
  select: (data) => data.length,
})
// Only re-renders when length changes
```

Memoize select for stability:
```tsx
const selectTodoCount = (data) => data.length
const { data } = useQuery({ queryKey: ['todos'], queryFn: fetchTodos, select: selectTodoCount })
```

## Window Focus Refetching

Disable globally:
```tsx
const queryClient = new QueryClient({
  defaultOptions: { queries: { refetchOnWindowFocus: false } },
})
```

Disable per-query:
```tsx
useQuery({ queryKey: ['todos'], queryFn: fetchTodos, refetchOnWindowFocus: false })
```

Custom focus events with `focusManager.setEventListener`.
React Native: Use `AppState` with `focusManager.setFocused`.

## Network Mode

- `'online'` (default): Queries/mutations don't fire without network. Retries pause offline.
- `'always'`: Ignores online/offline state. Good for AsyncStorage or non-network queries.
- `'offlineFirst'`: Run queryFn once, pause retries if offline. Good for service workers / HTTP caching.

## Background Fetching Indicators

Per-query:
```tsx
const { isFetching } = useQuery({ queryKey: ['todos'], queryFn: fetchTodos })
```

Global:
```tsx
import { useIsFetching } from '@tanstack/react-query'
const isFetching = useIsFetching()
```

## Prefetching

```tsx
// Basic prefetch
await queryClient.prefetchQuery({
  queryKey: ['todos'],
  queryFn: fetchTodos,
})

// Prefetch infinite query with multiple pages
await queryClient.prefetchInfiniteQuery({
  queryKey: ['projects'],
  queryFn: fetchProjects,
  initialPageParam: 0,
  getNextPageParam: (lastPage, pages) => lastPage.nextCursor,
  pages: 3,
})
```

**In event handlers**:
```tsx
const prefetch = () => {
  queryClient.prefetchQuery({
    queryKey: ['details'],
    queryFn: getDetailsData,
    staleTime: 60000,
  })
}
<button onMouseEnter={prefetch} onFocus={prefetch}>Show Details</button>
```

**In components** (flatten waterfalls):
```tsx
// Use useQuery and ignore result for prefetching
useQuery({
  queryKey: ['article-comments', id],
  queryFn: getArticleCommentsById,
  notifyOnChangeProps: [],
})

// Or with Suspense, use usePrefetchQuery
usePrefetchQuery({
  queryKey: ['article-comments', id],
  queryFn: getArticleCommentsById,
})
```

**Manual priming**:
```tsx
queryClient.setQueryData(['todos'], todos)
```
