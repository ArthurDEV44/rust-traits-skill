# TanStack Query Core Concepts

## Table of Contents
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Important Defaults](#important-defaults)
- [Queries](#queries)
- [Query Keys](#query-keys)
- [Query Functions](#query-functions)
- [Query Options](#query-options)
- [Mutations](#mutations)
- [Query Invalidation](#query-invalidation)
- [Invalidations from Mutations](#invalidations-from-mutations)
- [Updates from Mutation Responses](#updates-from-mutation-responses)
- [Filters](#filters)

## Installation

```bash
npm i @tanstack/react-query
# or: pnpm add / yarn add / bun add
```

Compatible with React v18+, ReactDOM and React Native.

Recommended ESLint plugin:
```bash
npm i -D @tanstack/eslint-plugin-query
```

## Quick Start

Three core concepts: Queries, Mutations, Query Invalidation.

```tsx
import {
  useQuery,
  useMutation,
  useQueryClient,
  QueryClient,
  QueryClientProvider,
} from '@tanstack/react-query'

const queryClient = new QueryClient()

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <Todos />
    </QueryClientProvider>
  )
}

function Todos() {
  const queryClient = useQueryClient()

  const query = useQuery({ queryKey: ['todos'], queryFn: getTodos })

  const mutation = useMutation({
    mutationFn: postTodo,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['todos'] })
    },
  })

  return (
    <div>
      <ul>
        {query.data?.map((todo) => (
          <li key={todo.id}>{todo.title}</li>
        ))}
      </ul>
      <button
        onClick={() => {
          mutation.mutate({ id: Date.now(), title: 'Do Laundry' })
        }}
      >
        Add Todo
      </button>
    </div>
  )
}
```

## Important Defaults

- Query instances via `useQuery`/`useInfiniteQuery` consider cached data as **stale** by default (`staleTime: 0`).
- Set `staleTime` to control freshness (e.g., `2 * 60 * 1000` for 2 min, `Infinity` for manual-only invalidation, `'static'` to never refetch even on invalidation).
- Stale queries refetch automatically on: new instance mount, window refocus, network reconnect.
- Customize with `refetchOnMount`, `refetchOnWindowFocus`, `refetchOnReconnect`.
- `refetchInterval` triggers periodic refetches independent of `staleTime`.
- Inactive queries are garbage collected after 5 min (`gcTime: 5 * 60 * 1000`).
- Failed queries retry **3 times with exponential backoff** by default.
- Results use **structural sharing** to preserve referential identity when data hasn't changed.

## Queries

A query is a declarative dependency on an async data source tied to a **unique key**.

```tsx
const { isPending, isError, data, error, isFetching } = useQuery({
  queryKey: ['todos'],
  queryFn: fetchTodoList,
})
```

**Status states** (mutually exclusive):
- `isPending` / `status === 'pending'` - No data yet
- `isError` / `status === 'error'` - Error occurred
- `isSuccess` / `status === 'success'` - Data available

**FetchStatus** (additional):
- `fetchStatus === 'fetching'` - Currently fetching
- `fetchStatus === 'paused'` - Wanted to fetch but paused (offline)
- `fetchStatus === 'idle'` - Not doing anything

**Why two states?** `status` = do we have data? `fetchStatus` = is queryFn running?

Pattern for rendering:
```tsx
if (isPending) return <span>Loading...</span>
if (isError) return <span>Error: {error.message}</span>
return <ul>{data.map(todo => <li key={todo.id}>{todo.title}</li>)}</ul>
```

## Query Keys

Must be an Array at top level. Serializable via `JSON.stringify` and unique to the query's data.

```tsx
// Simple
useQuery({ queryKey: ['todos'], ... })

// With variables
useQuery({ queryKey: ['todo', 5], ... })
useQuery({ queryKey: ['todo', 5, { preview: true }], ... })
useQuery({ queryKey: ['todos', { type: 'done' }], ... })
```

**Deterministic hashing**: Object key order doesn't matter, but array item order does.
```tsx
// These are equal:
useQuery({ queryKey: ['todos', { status, page }], ... })
useQuery({ queryKey: ['todos', { page, status }], ... })

// These are NOT equal:
useQuery({ queryKey: ['todos', status, page], ... })
useQuery({ queryKey: ['todos', page, status], ... })
```

**Include dependent variables** in the key:
```tsx
function Todos({ todoId }) {
  const result = useQuery({
    queryKey: ['todos', todoId],
    queryFn: () => fetchTodoById(todoId),
  })
}
```

## Query Functions

Any function returning a promise. Must resolve data or throw an error.

```tsx
useQuery({ queryKey: ['todos'], queryFn: fetchAllTodos })
useQuery({ queryKey: ['todos', todoId], queryFn: () => fetchTodoById(todoId) })
useQuery({
  queryKey: ['todos', todoId],
  queryFn: ({ queryKey }) => fetchTodoById(queryKey[1]),
})
```

**`fetch` does not throw by default** - throw manually:
```tsx
queryFn: async () => {
  const response = await fetch('/todos/' + todoId)
  if (!response.ok) throw new Error('Network response was not ok')
  return response.json()
}
```

**QueryFunctionContext**: `{ queryKey, client, signal?, meta? }` plus `pageParam` and `direction` for infinite queries.

## Query Options

Use `queryOptions` helper to share queryKey/queryFn between `useQuery`, `prefetchQuery`, etc. with full type inference:

```ts
import { queryOptions } from '@tanstack/react-query'

function groupOptions(id: number) {
  return queryOptions({
    queryKey: ['groups', id],
    queryFn: () => fetchGroups(id),
    staleTime: 5 * 1000,
  })
}

useQuery(groupOptions(1))
useSuspenseQuery(groupOptions(5))
queryClient.prefetchQuery(groupOptions(23))
queryClient.setQueryData(groupOptions(42).queryKey, newGroups)
```

Use `mutationOptions` similarly for mutations:
```ts
function groupMutationOptions() {
  return mutationOptions({
    mutationKey: ['addGroup'],
    mutationFn: addGroup,
  })
}
```

Override per-component:
```ts
const query = useQuery({
  ...groupOptions(1),
  select: (data) => data.groupName,
})
```

## Mutations

Used to create/update/delete data or perform server side-effects.

```tsx
const mutation = useMutation({
  mutationFn: (newTodo) => axios.post('/todos', newTodo),
})

// States: isIdle, isPending, isError, isSuccess
// Call: mutation.mutate({ id: Date.now(), title: 'Do Laundry' })
```

**Side effect callbacks**: `onMutate`, `onError`, `onSuccess`, `onSettled`

```tsx
useMutation({
  mutationFn: addTodo,
  onMutate: (variables, context) => {
    // Before mutation fires. Return rollback data.
    return { id: 1 }
  },
  onError: (error, variables, onMutateResult, context) => { /* rollback */ },
  onSuccess: (data, variables, onMutateResult, context) => { /* update cache */ },
  onSettled: (data, error, variables, onMutateResult, context) => { /* always */ },
})
```

**Promises**: Use `mutateAsync` for async/await:
```tsx
try {
  const data = await mutation.mutateAsync(todo)
} catch (error) { /* handle */ }
```

**Retry**: Mutations don't retry by default. Set `retry: 3` to enable.

**Scopes**: Mutations with same `scope.id` run in serial:
```tsx
useMutation({ mutationFn: addTodo, scope: { id: 'todo' } })
```

## Query Invalidation

Mark queries as stale and potentially refetch:

```tsx
queryClient.invalidateQueries() // all
queryClient.invalidateQueries({ queryKey: ['todos'] }) // prefix match
queryClient.invalidateQueries({ queryKey: ['todos'], exact: true }) // exact match
queryClient.invalidateQueries({
  predicate: (query) => query.queryKey[0] === 'todos' && query.queryKey[1]?.version >= 10,
})
```

When invalidated: marked stale (overrides staleTime) + refetched if currently rendered.

## Invalidations from Mutations

```tsx
const queryClient = useQueryClient()

const mutation = useMutation({
  mutationFn: addTodo,
  onSuccess: async () => {
    await queryClient.invalidateQueries({ queryKey: ['todos'] })
    // or multiple:
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: ['todos'] }),
      queryClient.invalidateQueries({ queryKey: ['reminders'] }),
    ])
  },
})
```

## Updates from Mutation Responses

Use `setQueryData` to update cache directly with mutation response:

```tsx
const mutation = useMutation({
  mutationFn: editTodo,
  onSuccess: (data) => {
    queryClient.setQueryData(['todo', { id: 5 }], data)
  },
})
```

**Immutability is required**:
```tsx
// Bad: oldData.title = 'new title'
// Good:
queryClient.setQueryData(['posts', { id }], (oldData) =>
  oldData ? { ...oldData, title: 'my new post title' } : oldData,
)
```

## Filters

### Query Filters
```tsx
queryClient.cancelQueries() // all
queryClient.removeQueries({ queryKey: ['posts'], type: 'inactive' })
queryClient.refetchQueries({ type: 'active' })
queryClient.refetchQueries({ queryKey: ['posts'], type: 'active' })
```

Properties: `queryKey?`, `exact?`, `type?: 'active' | 'inactive' | 'all'`, `stale?`, `fetchStatus?`, `predicate?`

### Mutation Filters
Properties: `mutationKey?`, `exact?`, `status?`, `predicate?`
