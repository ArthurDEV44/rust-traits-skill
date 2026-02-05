# TanStack Table Core Concepts

> Official docs: https://tanstack.com/table/latest/docs/introduction

## Overview

TanStack Table is a **headless UI** library for building tables & datagrids for TS/JS, React, Vue, Solid, Qwik, Svelte, Angular, and Lit. It provides logic, state, processing and APIs without any markup, styles, or pre-built implementations.

## Installation

Install only ONE adapter package:

```bash
# React (16.8+, 17, 18, 19)
npm install @tanstack/react-table

# Vue 3
npm install @tanstack/vue-table

# Solid-JS 1
npm install @tanstack/solid-table

# Svelte 3/4
npm install @tanstack/svelte-table

# Qwik 1
npm install @tanstack/qwik-table

# Angular 17+
npm install @tanstack/angular-table

# Lit 3
npm install @tanstack/lit-table

# Core (no framework)
npm install @tanstack/table-core
```

## Core Abstractions

- **Column Defs** - Objects to configure columns: data model, display templates, etc.
- **Table** - Core table object with state and APIs
- **Table Data** - The core data array (`TData[]`)
- **Columns** - Mirror column defs with column-specific APIs
- **Rows** - Mirror row data with row-specific APIs
- **Header Groups** - Computed slices of nested header levels
- **Headers** - Associated with or derived from column defs
- **Cells** - Row-column intersections with cell-specific APIs

## Data Setup

### Define TData Type

```ts
type User = {
  firstName: string
  lastName: string
  age: number
  visits: number
  progress: number
  status: string
}
```

### Stable References (Critical for React)

Data and columns MUST have stable references to prevent infinite re-renders:

```tsx
// GOOD
const [data, setData] = useState<User[]>([])
const columns = useMemo(() => [...], [])
const fallbackData: User[] = [] // defined outside component

// BAD - causes infinite re-renders
const data = [...] // redefined every render
const columns = [...] // redefined every render
```

### Deep Keyed Data

Access nested data with dot notation in `accessorKey` or `accessorFn`:

```ts
type User = { name: { first: string; last: string }; info: { age: number } }

const columns = [
  { header: 'First Name', accessorKey: 'name.first' },
  { header: 'Age', accessorFn: row => row.info.age },
]
```

### Sub-Row Data (for expanding)

```ts
type User = {
  firstName: string
  lastName: string
  subRows?: User[]
}
```

## Column Definitions

### Column Types

1. **Accessor Columns** - Have data model, can be sorted/filtered/grouped
2. **Display Columns** - No data model, for actions/checkboxes/expanders
3. **Grouping Columns** - Group other columns, define header/footer for groups

### Column Helper (recommended for TypeScript)

```tsx
const columnHelper = createColumnHelper<Person>()

const columns = [
  // Display Column
  columnHelper.display({
    id: 'actions',
    cell: props => <RowActions row={props.row} />,
  }),
  // Group Column
  columnHelper.group({
    header: 'Name',
    columns: [
      columnHelper.accessor('firstName', {
        cell: info => info.getValue(),
      }),
      columnHelper.accessor(row => row.lastName, {
        id: 'lastName',
        header: () => <span>Last Name</span>,
      }),
    ],
  }),
  // Accessor Column
  columnHelper.accessor('age', {
    header: () => 'Age',
  }),
]
```

### Accessor Types

```tsx
// Object key
columnHelper.accessor('firstName')
// or { accessorKey: 'firstName' }

// Array index
columnHelper.accessor(1)

// Accessor function (requires id)
columnHelper.accessor(row => `${row.firstName} ${row.lastName}`, { id: 'fullName' })
// or { id: 'fullName', accessorFn: row => `${row.firstName} ${row.lastName}` }
```

### Column ID Resolution

1. Object key / array index from accessor
2. Explicit `id` property
3. Primitive string `header` value

### Cell Rendering

```tsx
columnHelper.accessor('firstName', {
  cell: props => <span>{props.getValue().toUpperCase()}</span>,
})
// Access original row data:
columnHelper.accessor('firstName', {
  cell: props => <span>{`${props.row.original.id} - ${props.getValue()}`}</span>,
})
```

## Table Instance

### Creating

```tsx
// React
const table = useReactTable({ columns, data, getCoreRowModel: getCoreRowModel() })

// Vue
const table = useVueTable({ columns, data })

// Solid
const table = createSolidTable({ columns, data })

// Svelte
const table = createSvelteTable({ columns, data })
```

### Table State

```ts
table.getState().rowSelection // read state
table.setRowSelection(old => ({...old})) // set state
table.resetRowSelection() // reset state
```

### Rendering with flexRender

```tsx
import { flexRender } from '@tanstack/react-table'

// Headers
{table.getHeaderGroups().map(headerGroup => (
  <tr key={headerGroup.id}>
    {headerGroup.headers.map(header => (
      <th key={header.id} colSpan={header.colSpan}>
        {flexRender(header.column.columnDef.header, header.getContext())}
      </th>
    ))}
  </tr>
))}

// Body
{table.getRowModel().rows.map(row => (
  <tr key={row.id}>
    {row.getVisibleCells().map(cell => (
      <td key={cell.id}>
        {flexRender(cell.column.columnDef.cell, cell.getContext())}
      </td>
    ))}
  </tr>
))}
```

## Row Models

Import only what you need (tree-shakable):

```ts
import {
  getCoreRowModel,          // required - basic 1:1 data mapping
  getFilteredRowModel,      // client-side filtering
  getGroupedRowModel,       // grouping and aggregation
  getSortedRowModel,        // sorting
  getExpandedRowModel,      // expanding sub-rows
  getPaginationRowModel,    // client-side pagination
  getFacetedRowModel,       // faceted values (for filter UIs)
  getFacetedMinMaxValues,   // min/max faceted values
  getFacetedUniqueValues,   // unique faceted values
} from '@tanstack/react-table'
```

### Execution Order

`getCoreRowModel` → `getFilteredRowModel` → `getGroupedRowModel` → `getSortedRowModel` → `getExpandedRowModel` → `getPaginationRowModel` → `getRowModel`

### Row Model Data Structure

Each row model provides:
- `rows` - Array of rows
- `flatRows` - All sub-rows flattened to top level
- `rowsById` - Object keyed by row `id`

## Row APIs

- `row.id` - Unique ID (customize with `getRowId` table option)
- `row.getValue(columnId)` - Cached accessor value
- `row.renderValue(columnId)` - Like getValue but returns fallback if undefined
- `row.original` - Original unmodified data object
- `row.subRows` - Sub-rows array
- `row.depth` - Nesting depth (0 for root)
- `row.parentId` / `row.getParentRow()` - Parent row reference

## Cell APIs

- `cell.id` - `${row.id}_${column.id}`
- `cell.getValue()` / `cell.renderValue()` - Shortcuts for row.getValue/renderValue
- `cell.row` - Parent row reference
- `cell.column` - Parent column reference
- `cell.getContext()` - Rendering context for flexRender

## Header APIs

- `header.id` - Unique identifier
- `header.column` - Associated Column object
- `header.headerGroup` - Parent HeaderGroup
- `header.colSpan` / `header.rowSpan` - Span values
- `header.isPlaceholder` - True for placeholder headers
- `header.subHeaders` - Child headers array
- `header.getContext()` - Rendering context for flexRender
