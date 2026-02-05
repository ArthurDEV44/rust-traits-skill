---
name: tanstack-table
description: "TanStack Table v8 headless UI library for building type-safe tables and datagrids in React, Vue, Solid, Svelte, Qwik, Angular, and Lit. Use when writing, reviewing, or refactoring code involving TanStack Table: (1) Creating tables with useReactTable, createColumnHelper, flexRender, (2) Implementing sorting, filtering, pagination, grouping, expanding, row selection, column pinning, column sizing/resizing, column ordering, column visibility, (3) Managing table state with onSortingChange, onPaginationChange, etc., (4) Using row models (getCoreRowModel, getSortedRowModel, getFilteredRowModel, getPaginationRowModel), (5) Server-side operations with manualSorting/manualFiltering/manualPagination, (6) Migrating from react-table v7 to @tanstack/react-table v8, (7) Custom features with _features option, column faceting, fuzzy filtering, virtualization."
---

# TanStack Table v8

Headless UI library for building tables and datagrids. Provides state management, data processing, and APIs without markup or styles.

## Quick Start (React)

```tsx
import { useReactTable, getCoreRowModel, flexRender, createColumnHelper } from '@tanstack/react-table'

type Person = { firstName: string; lastName: string; age: number }

const columnHelper = createColumnHelper<Person>()

const columns = [
  columnHelper.accessor('firstName', { header: 'First Name', cell: info => info.getValue() }),
  columnHelper.accessor('lastName', { header: 'Last Name' }),
  columnHelper.accessor('age', { header: 'Age' }),
]

function MyTable({ data }: { data: Person[] }) {
  const table = useReactTable({ data, columns, getCoreRowModel: getCoreRowModel() })

  return (
    <table>
      <thead>
        {table.getHeaderGroups().map(hg => (
          <tr key={hg.id}>
            {hg.headers.map(h => (
              <th key={h.id} colSpan={h.colSpan}>
                {flexRender(h.column.columnDef.header, h.getContext())}
              </th>
            ))}
          </tr>
        ))}
      </thead>
      <tbody>
        {table.getRowModel().rows.map(row => (
          <tr key={row.id}>
            {row.getVisibleCells().map(cell => (
              <td key={cell.id}>{flexRender(cell.column.columnDef.cell, cell.getContext())}</td>
            ))}
          </tr>
        ))}
      </tbody>
    </table>
  )
}
```

## Critical Rules

1. **Stable references**: `data` and `columns` MUST use `useState`, `useMemo`, or be defined outside the component. Inline arrays cause infinite re-renders.
2. **Import only needed row models**: Tree-shakable. Only import `getCoreRowModel`, `getSortedRowModel`, etc. as needed.
3. **Use `flexRender`** for rendering headers and cells (handles string, JSX, and function column defs).
4. **Use `getVisibleCells()`** (not `getAllCells()`) when column visibility is used.
5. **Row model execution order**: Core → Filtered → Grouped → Sorted → Expanded → Paginated.

## Feature Quick Reference

| Feature | Row Model Import | Key State | Key API |
|---------|-----------------|-----------|---------|
| Sorting | `getSortedRowModel` | `SortingState` | `column.getToggleSortingHandler()` |
| Column Filter | `getFilteredRowModel` | `ColumnFiltersState` | `column.setFilterValue()` |
| Global Filter | `getFilteredRowModel` | `globalFilter` | `table.setGlobalFilter()` |
| Pagination | `getPaginationRowModel` | `PaginationState` | `table.nextPage()` / `table.previousPage()` |
| Row Selection | (none) | `RowSelectionState` | `row.getToggleSelectedHandler()` |
| Expanding | `getExpandedRowModel` | `ExpandedState` | `row.getToggleExpandedHandler()` |
| Grouping | `getGroupedRowModel` | `GroupingState` | `table.setGrouping()` |
| Col Visibility | (none) | `ColumnVisibilityState` | `column.getToggleVisibilityHandler()` |
| Col Pinning | (none) | `ColumnPinningState` | `column.pin()` |
| Col Sizing | (none) | `ColumnSizingState` | `header.getResizeHandler()` |
| Col Ordering | (none) | `columnOrder: string[]` | `table.setColumnOrder()` |
| Faceting | `getFacetedRowModel` + `getFacetedUniqueValues`/`getFacetedMinMaxValues` | (none) | `column.getFacetedUniqueValues()` |

## Server-Side Operations

Disable client-side processing with `manual*` options:

```tsx
const table = useReactTable({
  data, columns,
  getCoreRowModel: getCoreRowModel(),
  manualSorting: true,      // data is pre-sorted
  manualFiltering: true,    // data is pre-filtered
  manualPagination: true,   // data is pre-paginated
  rowCount: totalRows,      // tell table total count for pagination
  state: { sorting, columnFilters, pagination },
  onSortingChange: setSorting,
  onColumnFiltersChange: setColumnFilters,
  onPaginationChange: setPagination,
})
```

## References

- **Core concepts, data setup, column defs, table instance, row models, APIs**: See [references/core-concepts.md](references/core-concepts.md)
- **All features (sorting, filtering, pagination, selection, pinning, sizing, grouping, expanding, faceting, virtualization, custom features)**: See [references/features.md](references/features.md)
- **State management patterns (controlled, uncontrolled, fully controlled, server-side)**: See [references/state-management.md](references/state-management.md)
- **Migration from react-table v7 to v8**: See [references/migration-v8.md](references/migration-v8.md)

## Official Documentation Links

- Introduction: https://tanstack.com/table/latest/docs/introduction
- Installation: https://tanstack.com/table/latest/docs/installation
- Overview: https://tanstack.com/table/latest/docs/guide/overview
- Column Defs Guide: https://tanstack.com/table/latest/docs/guide/column-defs
- Table Instance Guide: https://tanstack.com/table/latest/docs/guide/tables
- Row Models Guide: https://tanstack.com/table/latest/docs/guide/row-models
- State Guide: https://tanstack.com/table/latest/docs/guide/table-state
- Sorting: https://tanstack.com/table/latest/docs/guide/sorting
- Filtering: https://tanstack.com/table/latest/docs/guide/column-filtering
- Global Filtering: https://tanstack.com/table/latest/docs/guide/global-filtering
- Pagination: https://tanstack.com/table/latest/docs/guide/pagination
- Row Selection: https://tanstack.com/table/latest/docs/guide/row-selection
- Expanding: https://tanstack.com/table/latest/docs/guide/expanding
- Grouping: https://tanstack.com/table/latest/docs/guide/grouping
- Column Visibility: https://tanstack.com/table/latest/docs/guide/column-visibility
- Column Ordering: https://tanstack.com/table/latest/docs/guide/column-ordering
- Column Pinning: https://tanstack.com/table/latest/docs/guide/column-pinning
- Column Sizing: https://tanstack.com/table/latest/docs/guide/column-sizing
- Faceting: https://tanstack.com/table/latest/docs/guide/column-faceting
- Virtualization: https://tanstack.com/table/latest/docs/guide/virtualization
- Custom Features: https://tanstack.com/table/latest/docs/guide/custom-features
- Migration v8: https://tanstack.com/table/latest/docs/guide/migrating
- API - Table: https://tanstack.com/table/latest/docs/api/core/table
- API - Column Def: https://tanstack.com/table/latest/docs/api/core/column-def
- API - Column: https://tanstack.com/table/latest/docs/api/core/column
- API - Row: https://tanstack.com/table/latest/docs/api/core/row
- API - Cell: https://tanstack.com/table/latest/docs/api/core/cell
- API - Header: https://tanstack.com/table/latest/docs/api/core/header
- API - Header Group: https://tanstack.com/table/latest/docs/api/core/header-group
- React Adapter: https://tanstack.com/table/latest/docs/framework/react/react-table
- Examples: https://tanstack.com/table/latest/docs/framework/react/examples/basic
