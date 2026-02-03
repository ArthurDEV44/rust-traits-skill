# TanStack Query TypeScript Guide

## Table of Contents
- [Type Inference](#type-inference)
- [Type Narrowing](#type-narrowing)
- [Typing Errors](#typing-errors)
- [Registering Global Types](#registering-global-types)
- [Typing Query Options](#typing-query-options)
- [skipToken for Typesafe Disabling](#skiptoken-for-typesafe-disabling)
- [GraphQL Integration](#graphql-integration)

## Type Inference

Types flow through automatically from `queryFn`:

```tsx
const { data } = useQuery({
  //    ^? const data: number | undefined
  queryKey: ['test'],
  queryFn: () => Promise.resolve(5),
})
```

With `select`:
```tsx
const { data } = useQuery({
  //    ^? const data: string | undefined
  queryKey: ['test'],
  queryFn: () => Promise.resolve(5),
  select: (data) => data.toString(),
})
```

Ensure `queryFn` has a well-defined return type (most fetch libraries return `any`):
```tsx
const fetchGroups = (): Promise<Group[]> =>
  axios.get('/groups').then((response) => response.data)

const { data } = useQuery({ queryKey: ['groups'], queryFn: fetchGroups })
//      ^? const data: Group[] | undefined
```

## Type Narrowing

Uses discriminated union on `status` field:
```tsx
const { data, isSuccess } = useQuery({
  queryKey: ['test'],
  queryFn: () => Promise.resolve(5),
})

if (isSuccess) {
  data // ^? const data: number
}
```

## Typing Errors

Default error type is `Error`. For custom error types:
```tsx
const { error } = useQuery<Group[], string>(['groups'], fetchGroups)
//      ^? const error: string | null
```

Better: use type narrowing with `AxiosError`:
```tsx
const { error } = useQuery({ queryKey: ['groups'], queryFn: fetchGroups })
if (axios.isAxiosError(error)) {
  error // ^? const error: AxiosError
}
```

### Register Global Error Type

```tsx
import '@tanstack/react-query'

declare module '@tanstack/react-query' {
  interface Register {
    defaultError: unknown // or AxiosError, etc.
  }
}
```

## Registering Global Types

### Global Meta
```ts
import '@tanstack/react-query'

interface MyMeta extends Record<string, unknown> {
  // your fields
}

declare module '@tanstack/react-query' {
  interface Register {
    queryMeta: MyMeta
    mutationMeta: MyMeta
  }
}
```

### Global Query/Mutation Keys
```ts
import '@tanstack/react-query'

type QueryKey = ['dashboard' | 'marketing', ...ReadonlyArray<unknown>]

declare module '@tanstack/react-query' {
  interface Register {
    queryKey: QueryKey
    mutationKey: QueryKey
  }
}
```

## Typing Query Options

Use `queryOptions` helper for shared options with full inference:
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
queryClient.prefetchQuery(groupOptions(23))
const data = queryClient.getQueryData(groupOptions().queryKey)
//     ^? const data: Group[] | undefined
```

Use `mutationOptions` similarly:
```ts
function groupMutationOptions() {
  return mutationOptions({
    mutationKey: ['addGroup'],
    mutationFn: addGroup,
  })
}
```

## skipToken for Typesafe Disabling

```tsx
import { skipToken, useQuery } from '@tanstack/react-query'

function Todos() {
  const [filter, setFilter] = React.useState<string | undefined>()

  const { data } = useQuery({
    queryKey: ['todos', filter],
    queryFn: filter ? () => fetchTodos(filter) : skipToken,
  })
}
```

Note: `refetch()` does not work with `skipToken`. Use `enabled: false` if you need manual refetch.

## GraphQL Integration

With `graphql-request` v5+ and GraphQL Code Generator:

```tsx
import request from 'graphql-request'
import { useQuery } from '@tanstack/react-query'
import { graphql } from './gql/gql'

const allFilmsQuery = graphql(`
  query allFilmsWithVariablesQuery($first: Int!) {
    allFilms(first: $first) {
      edges { node { id, title } }
    }
  }
`)

function App() {
  const { data } = useQuery({
    queryKey: ['films'],
    queryFn: async () =>
      request(
        'https://swapi-graphql.netlify.app/.netlify/functions/index',
        allFilmsQuery,
        { first: 10 }, // variables are type-checked
      ),
  })
}
```
