# TanStack Table State Management (React)

> Official docs: https://tanstack.com/table/latest/docs/guide/table-state

## Internal State (Default)

By default, the table manages all state internally. Access it via `table.getState()`:

```tsx
const table = useReactTable({ columns, data })
console.log(table.getState()) // entire state
console.log(table.getState().sorting) // specific state
```

## Initial State

Set initial values without managing state yourself:

```tsx
const table = useReactTable({
  columns, data,
  initialState: {
    columnOrder: ['age', 'firstName', 'lastName'],
    columnVisibility: { id: false },
    expanded: true,
    sorting: [{ id: 'age', desc: true }],
    pagination: { pageIndex: 0, pageSize: 25 },
  },
})
```

> Do NOT pass the same state to both `initialState` and `state`. `state` takes precedence.

## Controlled State (Individual)

Control only the state you need. Requires both `state.X` and `onXChange`:

```tsx
const [sorting, setSorting] = useState<SortingState>([])
const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>([])
const [pagination, setPagination] = useState({ pageIndex: 0, pageSize: 15 })

const table = useReactTable({
  columns, data,
  state: { sorting, columnFilters, pagination },
  onSortingChange: setSorting,
  onColumnFiltersChange: setColumnFilters,
  onPaginationChange: setPagination,
})
```

## Fully Controlled State

Control entire state with `onStateChange`:

```tsx
const table = useReactTable({ columns, data })
const [state, setState] = useState({ ...table.initialState })

table.setOptions(prev => ({
  ...prev,
  state,
  onStateChange: setState,
}))
```

## Updater Callbacks

`on[State]Change` callbacks receive either a value or an updater function:

```tsx
onSortingChange: (updater) => {
  const newValue = updater instanceof Function ? updater(sorting) : updater
  // custom logic here
  setSorting(newValue)
}
```

## State Types

Import TypeScript types for type-safe state:

```tsx
import { type SortingState, type ColumnFiltersState, type PaginationState } from '@tanstack/react-table'
```

## Common Pattern: Server-Side Data Fetching

```tsx
const [sorting, setSorting] = useState<SortingState>([])
const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>([])
const [pagination, setPagination] = useState({ pageIndex: 0, pageSize: 15 })

const query = useQuery({
  queryKey: ['data', sorting, columnFilters, pagination],
  queryFn: () => fetchData(sorting, columnFilters, pagination),
})

const table = useReactTable({
  columns,
  data: query.data?.rows ?? [],
  manualSorting: true,
  manualFiltering: true,
  manualPagination: true,
  rowCount: query.data?.totalCount,
  state: { sorting, columnFilters, pagination },
  onSortingChange: setSorting,
  onColumnFiltersChange: setColumnFilters,
  onPaginationChange: setPagination,
  getCoreRowModel: getCoreRowModel(),
})
```
