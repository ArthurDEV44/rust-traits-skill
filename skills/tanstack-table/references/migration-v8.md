# Migrating from React Table v7 to TanStack Table v8

> Official docs: https://tanstack.com/table/latest/docs/guide/migrating

## Install

```bash
npm uninstall react-table @types/react-table
npm install @tanstack/react-table
```

Types are included in the base package.

## Key Changes

### Hook & Plugin System → Row Model Imports

```tsx
// v7
import { useTable, useSortBy, usePagination } from 'react-table'
const table = useTable({ columns, data }, useSortBy, usePagination)

// v8
import { useReactTable, getCoreRowModel, getSortedRowModel, getPaginationRowModel } from '@tanstack/react-table'
const table = useReactTable({
  columns, data,
  getCoreRowModel: getCoreRowModel(),
  getSortedRowModel: getSortedRowModel(),
  getPaginationRowModel: getPaginationRowModel(),
})
```

### Table Options

- `disable*` → `enable*` (e.g., `disableSortBy` → `enableSorting`)

### Column Definitions

- `accessor` → `accessorKey` (string) or `accessorFn` (function)
- `Header` → `header`, `Cell` → `cell`, `Footer` → `footer`
- `width`/`minWidth`/`maxWidth` → `size`/`minSize`/`maxSize`
- `sortType` → `sortingFn`
- `disable*` → `enable*`
- `value` → `getValue()` (lazy evaluation with caching)
- `cell.isGrouped` → `cell.getIsGrouped()` (etc.)

### Column Helper (optional, recommended for TS)

```tsx
const columnHelper = createColumnHelper<Person>()
columnHelper.accessor('firstName', { header: 'First Name' })
columnHelper.accessor(row => row.lastName, { header: () => <span>Last Name</span> })
```

### Rendering

```tsx
// v7
<th {...header.getHeaderProps()}>{cell.render('Header')}</th>
<td {...cell.getCellProps()}>{cell.render('Cell')}</td>

// v8
<th colSpan={header.colSpan} key={column.id}>
  {flexRender(header.column.columnDef.header, header.getContext())}
</th>
<td key={cell.id}>
  {flexRender(cell.column.columnDef.cell, cell.getContext())}
</td>
```

### Row Selection

```tsx
// v7
<input type="checkbox" {...row.getToggleRowSelectedProps()} />

// v8
<input type="checkbox"
  checked={row.getIsSelected()}
  disabled={!row.getCanSelect()}
  onChange={row.getToggleSelectedHandler()}
/>
```

### Filter Functions

```tsx
// v7 - returns filtered rows array
(rows: Row[], id: string, filterValue: any) => Row[]

// v8 - returns boolean per row
(row: Row, id: string, filterValue: any) => boolean
```

### Other

- No default `style` or `role` attributes provided anymore (headless)
- Must manually define `key`, `onClick`, `colSpan` props
- New Dev Tools available
