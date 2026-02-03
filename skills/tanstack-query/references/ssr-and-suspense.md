# TanStack Query SSR, Suspense & Server Components

## Table of Contents
- [Suspense](#suspense)
- [Server Rendering & Hydration](#server-rendering--hydration)
- [Advanced Server Rendering (App Router / Server Components)](#advanced-server-rendering)
- [Testing](#testing)

## Suspense

Dedicated hooks: `useSuspenseQuery`, `useSuspenseInfiniteQuery`, `useSuspenseQueries`.

```tsx
import { useSuspenseQuery } from '@tanstack/react-query'

const { data } = useSuspenseQuery({ queryKey, queryFn })
// data is guaranteed defined (errors/loading handled by Suspense/ErrorBoundaries)
```

- Cannot conditionally enable/disable suspense queries.
- Use `startTransition` to prevent fallback during key changes.
- `throwOnError` default: only throws if no cached data exists.

### Error Boundaries

```tsx
import { QueryErrorResetBoundary } from '@tanstack/react-query'
import { ErrorBoundary } from 'react-error-boundary'

<QueryErrorResetBoundary>
  {({ reset }) => (
    <ErrorBoundary
      onReset={reset}
      fallbackRender={({ resetErrorBoundary }) => (
        <div>
          Error! <button onClick={resetErrorBoundary}>Try again</button>
        </div>
      )}
    >
      <Page />
    </ErrorBoundary>
  )}
</QueryErrorResetBoundary>
```

### Experimental: useQuery().promise with React.use()

Enable with `experimental_prefetchInRender: true` on QueryClient:

```tsx
function TodoList({ query }: { query: UseQueryResult<Todo[]> }) {
  const data = React.use(query.promise)
  return <ul>{data.map(todo => <li key={todo.id}>{todo.title}</li>)}</ul>
}

export function App() {
  const query = useQuery({ queryKey: ['todos'], queryFn: fetchTodos })
  return (
    <React.Suspense fallback={<div>Loading...</div>}>
      <TodoList query={query} />
    </React.Suspense>
  )
}
```

## Server Rendering & Hydration

### Setup (Pages Router / Remix)

Create `queryClient` inside the app in React state:
```tsx
// _app.tsx (Next.js) or app/root.tsx (Remix)
export default function MyApp({ Component, pageProps }) {
  const [queryClient] = React.useState(
    () => new QueryClient({
      defaultOptions: {
        queries: { staleTime: 60 * 1000 }, // avoid immediate refetch on client
      },
    }),
  )
  return (
    <QueryClientProvider client={queryClient}>
      <Component {...pageProps} />
    </QueryClientProvider>
  )
}
```

**Never** create queryClient at file root level (leaks data between users).

### Hydration Pattern (Recommended)

In loader/getStaticProps/getServerSideProps:
```tsx
export async function getStaticProps() {
  const queryClient = new QueryClient()
  await queryClient.prefetchQuery({
    queryKey: ['posts'],
    queryFn: getPosts,
  })
  return { props: { dehydratedState: dehydrate(queryClient) } }
}
```

In route component:
```tsx
export default function PostsRoute({ dehydratedState }) {
  return (
    <HydrationBoundary state={dehydratedState}>
      <Posts />
    </HydrationBoundary>
  )
}
```

Or remove boilerplate by placing `HydrationBoundary` in `_app.tsx`:
```tsx
<QueryClientProvider client={queryClient}>
  <HydrationBoundary state={pageProps.dehydratedState}>
    <Component {...pageProps} />
  </HydrationBoundary>
</QueryClientProvider>
```

### Quick Alternative: initialData

```tsx
export async function getServerSideProps() {
  const posts = await getPosts()
  return { props: { posts } }
}

function Posts(props) {
  const { data } = useQuery({
    queryKey: ['posts'],
    queryFn: getPosts,
    initialData: props.posts,
  })
}
```

Drawbacks: must pass down to every component, `dataUpdatedAt` is based on page load time, doesn't overwrite stale cache data.

### Prefetching Dependent Queries on Server

```tsx
export async function getServerSideProps() {
  const queryClient = new QueryClient()
  const user = await queryClient.fetchQuery({
    queryKey: ['user', email],
    queryFn: getUserByEmail,
  })
  if (user?.userId) {
    await queryClient.prefetchQuery({
      queryKey: ['projects', userId],
      queryFn: getProjectsByUser,
    })
  }
  return { props: { dehydratedState: dehydrate(queryClient) } }
}
```

### Error Handling

- `prefetchQuery` never throws (graceful degradation).
- `dehydrate` only includes successful queries by default.
- Use `fetchQuery` to throw on errors for critical content.
- Override with `shouldDehydrateQuery: (query) => true` to include failed queries.

## Advanced Server Rendering

### App Router Setup (Next.js)

Providers (Client Component):
```tsx
// app/providers.tsx
'use client'
import { isServer, QueryClient, QueryClientProvider } from '@tanstack/react-query'

function makeQueryClient() {
  return new QueryClient({
    defaultOptions: { queries: { staleTime: 60 * 1000 } },
  })
}

let browserQueryClient: QueryClient | undefined

function getQueryClient() {
  if (isServer) return makeQueryClient()
  if (!browserQueryClient) browserQueryClient = makeQueryClient()
  return browserQueryClient
}

export default function Providers({ children }: { children: React.ReactNode }) {
  const queryClient = getQueryClient()
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
}
```

Layout:
```tsx
// app/layout.tsx
import Providers from './providers'

export default function RootLayout({ children }) {
  return (
    <html lang="en">
      <body><Providers>{children}</Providers></body>
    </html>
  )
}
```

### Prefetching in Server Components

```tsx
// app/posts/page.tsx (Server Component)
import { dehydrate, HydrationBoundary, QueryClient } from '@tanstack/react-query'
import Posts from './posts'

export default async function PostsPage() {
  const queryClient = new QueryClient()
  await queryClient.prefetchQuery({
    queryKey: ['posts'],
    queryFn: getPosts,
  })
  return (
    <HydrationBoundary state={dehydrate(queryClient)}>
      <Posts />
    </HydrationBoundary>
  )
}
```

```tsx
// app/posts/posts.tsx (Client Component)
'use client'
export default function Posts() {
  const { data } = useQuery({ queryKey: ['posts'], queryFn: getPosts })
  // ...
}
```

### Streaming with Server Components

Dehydrate pending queries for streaming:
```tsx
// app/get-query-client.ts
import { QueryClient, defaultShouldDehydrateQuery } from '@tanstack/react-query'

function makeQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: { staleTime: 60 * 1000 },
      dehydrate: {
        shouldDehydrateQuery: (query) =>
          defaultShouldDehydrateQuery(query) || query.state.status === 'pending',
        shouldRedactErrors: () => false,
      },
    },
  })
}
```

Then prefetch without `await`:
```tsx
export default function PostsPage() {
  const queryClient = getQueryClient()
  queryClient.prefetchQuery({ queryKey: ['posts'], queryFn: getPosts }) // no await!
  return (
    <HydrationBoundary state={dehydrate(queryClient)}>
      <Posts />
    </HydrationBoundary>
  )
}
```

Use `useSuspenseQuery` in client component to consume the streamed data.

### Experimental: Streaming without Prefetching

`@tanstack/react-query-next-experimental`:
```tsx
import { ReactQueryStreamedHydration } from '@tanstack/react-query-next-experimental'

<QueryClientProvider client={queryClient}>
  <ReactQueryStreamedHydration>
    {children}
  </ReactQueryStreamedHydration>
</QueryClientProvider>
```

Allows `useSuspenseQuery` in Client Components without explicit prefetching. Results stream from server as SuspenseBoundaries resolve. Tradeoff: creates request waterfalls on page navigations.

### Data Ownership Warning

Avoid rendering prefetched data directly in Server Components AND in Client Components via useQuery. The Server Component can't revalidate, causing stale UI. Treat Server Components as a place to prefetch data, not to render query results.

## Testing

```tsx
const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false } },
})
const wrapper = ({ children }) => (
  <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
)

const { result } = renderHook(() => useCustomHook(), { wrapper })
await waitFor(() => expect(result.current.isSuccess).toBe(true))
expect(result.current.data).toEqual('Hello')
```

Key tips:
- Turn off retries for tests.
- Set `gcTime: Infinity` for Jest to avoid "did not exit" errors.
- Use `nock` or `msw` for mocking network requests.
- Use `waitFor` for async assertions.
