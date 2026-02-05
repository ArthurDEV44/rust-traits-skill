# TanStack Table Features Guide

> Official docs: https://tanstack.com/table/latest/docs/guide/sorting

## Table of Contents

- [Sorting](#sorting)
- [Column Filtering](#column-filtering)
- [Global Filtering](#global-filtering)
- [Fuzzy Filtering](#fuzzy-filtering)
- [Pagination](#pagination)
- [Row Selection](#row-selection)
- [Column Visibility](#column-visibility)
- [Column Ordering](#column-ordering)
- [Column Pinning](#column-pinning)
- [Column Sizing & Resizing](#column-sizing--resizing)
- [Expanding](#expanding)
- [Grouping](#grouping)
- [Column Faceting](#column-faceting)
- [Row Pinning](#row-pinning)
- [Virtualization](#virtualization)
- [Custom Features](#custom-features)

---

## Sorting

> API: https://tanstack.com/table/latest/docs/api/features/sorting

### Setup

```tsx
import { getSortedRowModel } from '@tanstack/react-table'

const table = useReactTable({
  data, columns,
  getCoreRowModel: getCoreRowModel(),
  getSortedRowModel: getSortedRowModel(),
})
```

### State

```ts
type SortingState = { id: string; desc: boolean }[]

const [sorting, setSorting] = useState<SortingState>([])
// Pass to table:
state: { sorting },
onSortingChange: setSorting,
```

### Built-in Sorting Functions

- `alphanumeric` / `alphanumericCaseSensitive` - Mixed alphanumeric (slower, more accurate)
- `text` / `textCaseSensitive` - String values (faster)
- `datetime` - Date objects
- `basic` - Simple `a > b` comparison (fastest)

### Custom Sorting Function

```ts
const mySort: SortingFn<TData> = (rowA, rowB, columnId) => {
  return // -1, 0, or 1 in ascending order
}
// In column def:
{ accessorKey: 'field', sortingFn: mySort }
// Or as global:
sortingFns: { mySort }
```

### Key Options

- `enableSorting` (table/column) - Enable/disable sorting
- `sortDescFirst` (table/column) - First sort direction
- `invertSorting` (column) - Invert sort order (for rankings)
- `sortUndefined` (column) - `'first'` | `'last'` | `false` | `-1` | `1`
- `enableSortingRemoval` (table) - Allow removing sort (default: true)
- `enableMultiSort` (table/column) - Multi-column sorting (Shift+click)
- `maxMultiSortColCount` (table) - Limit multi-sort columns
- `manualSorting` (table) - Disable client-side sorting for server-side

### Key APIs

- `column.getToggleSortingHandler()` - Click handler for sorting UI
- `column.getIsSorted()` - Returns `'asc'` | `'desc'` | `false`
- `column.toggleSorting(desc?, multi?)` - Toggle sorting programmatically

---

## Column Filtering

> API: https://tanstack.com/table/latest/docs/api/features/column-filtering

### Setup

```tsx
import { getFilteredRowModel } from '@tanstack/react-table'

const table = useReactTable({
  data, columns,
  getCoreRowModel: getCoreRowModel(),
  getFilteredRowModel: getFilteredRowModel(),
})
```

### State

```ts
type ColumnFiltersState = { id: string; value: unknown }[]

const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>([])
state: { columnFilters },
onColumnFiltersChange: setColumnFilters,
```

### Built-in Filter Functions

- `includesString` / `includesStringSensitive`
- `equalsString` / `equalsStringSensitive`
- `arrIncludes` / `arrIncludesAll` / `arrIncludesSome`
- `equals` / `weakEquals`
- `inNumberRange`

### Custom Filter Function

```ts
const myFilter: FilterFn = (row, columnId, filterValue, addMeta) => {
  return true // or false
}
// Attach optional behaviors:
myFilter.autoRemove = (val) => !val
myFilter.resolveFilterValue = (val) => val.toString().toLowerCase().trim()
```

### Key Options

- `filterFn` (column) - Specify filter function per column
- `enableColumnFilter` (column) / `enableColumnFilters` (table)
- `manualFiltering` (table) - For server-side filtering
- `filterFromLeafRows` (table) - Filter from leaf rows up (for expanding)
- `maxLeafRowFilterDepth` (table) - Max depth for leaf row filtering

### Key APIs

- `column.getFilterValue()` / `column.setFilterValue(value)`
- `column.getCanFilter()` / `column.getIsFiltered()`

---

## Global Filtering

> API: https://tanstack.com/table/latest/docs/api/features/global-filtering

### Setup

Same `getFilteredRowModel` as column filtering. Set `globalFilterFn` for the filter function.

```tsx
const [globalFilter, setGlobalFilter] = useState('')

const table = useReactTable({
  data, columns,
  getCoreRowModel: getCoreRowModel(),
  getFilteredRowModel: getFilteredRowModel(),
  globalFilterFn: 'includesString',
  state: { globalFilter },
  onGlobalFilterChange: setGlobalFilter,
})
```

### UI

```tsx
<input
  value={globalFilter ?? ''}
  onChange={e => table.setGlobalFilter(String(e.target.value))}
  placeholder="Search..."
/>
```

---

## Fuzzy Filtering

> Example: https://tanstack.com/table/latest/docs/framework/react/examples/filters-fuzzy

Requires `@tanstack/match-sorter-utils`:

```tsx
import { rankItem } from '@tanstack/match-sorter-utils'

const fuzzyFilter: FilterFn<any> = (row, columnId, value, addMeta) => {
  const itemRank = rankItem(row.getValue(columnId), value)
  addMeta({ itemRank })
  return itemRank.passed
}

// Optional: fuzzy sort by rank
const fuzzySort: SortingFn<any> = (rowA, rowB, columnId) => {
  let dir = 0
  if (rowA.columnFiltersMeta[columnId]) {
    dir = compareItems(
      rowA.columnFiltersMeta[columnId]?.itemRank!,
      rowB.columnFiltersMeta[columnId]?.itemRank!
    )
  }
  return dir === 0 ? sortingFns.alphanumeric(rowA, rowB, columnId) : dir
}
```

---

## Pagination

> API: https://tanstack.com/table/latest/docs/api/features/pagination

### Client-Side

```tsx
import { getPaginationRowModel } from '@tanstack/react-table'

const [pagination, setPagination] = useState({ pageIndex: 0, pageSize: 10 })

const table = useReactTable({
  data, columns,
  getCoreRowModel: getCoreRowModel(),
  getPaginationRowModel: getPaginationRowModel(),
  state: { pagination },
  onPaginationChange: setPagination,
})
```

### Server-Side

```tsx
const table = useReactTable({
  data, columns,
  getCoreRowModel: getCoreRowModel(),
  manualPagination: true,
  rowCount: totalRowCount, // or pageCount
  state: { pagination },
  onPaginationChange: setPagination,
})
```

### Key APIs

- `table.firstPage()` / `table.lastPage()` / `table.previousPage()` / `table.nextPage()`
- `table.getCanPreviousPage()` / `table.getCanNextPage()`
- `table.setPageIndex(index)` / `table.setPageSize(size)`
- `table.getPageCount()` / `table.getRowCount()`

### Options

- `autoResetPageIndex` (table) - Reset pageIndex on data changes (default: true, auto-disabled with manualPagination)

---

## Row Selection

> API: https://tanstack.com/table/latest/docs/api/features/row-selection

### State

```ts
const [rowSelection, setRowSelection] = useState<RowSelectionState>({})

const table = useReactTable({
  data, columns,
  getCoreRowModel: getCoreRowModel(),
  state: { rowSelection },
  onRowSelectionChange: setRowSelection,
  getRowId: row => row.uuid, // recommended for meaningful IDs
})
```

### Checkbox Column

```tsx
columnHelper.display({
  id: 'select',
  header: ({ table }) => (
    <input type="checkbox"
      checked={table.getIsAllRowsSelected()}
      onChange={table.getToggleAllRowsSelectedHandler()}
    />
  ),
  cell: ({ row }) => (
    <input type="checkbox"
      checked={row.getIsSelected()}
      disabled={!row.getCanSelect()}
      onChange={row.getToggleSelectedHandler()}
    />
  ),
})
```

### Key Options

- `enableRowSelection` (table) - Boolean or `(row) => boolean`
- `enableMultiRowSelection` (table) - Set `false` for single selection (radio buttons)
- `enableSubRowSelection` (table) - Auto-select sub-rows with parent

### Key APIs

- `table.getSelectedRowModel().rows` - Selected rows
- `table.getFilteredSelectedRowModel().rows` - Filtered selected rows
- `row.getIsSelected()` / `row.toggleSelected()`

---

## Column Visibility

> API: https://tanstack.com/table/latest/docs/api/features/column-visibility

### State

```ts
const [columnVisibility, setColumnVisibility] = useState({ colId: false })

state: { columnVisibility },
onColumnVisibilityChange: setColumnVisibility,
```

### Toggle UI

```tsx
{table.getAllColumns().map(column => (
  <label key={column.id}>
    <input
      checked={column.getIsVisible()}
      disabled={!column.getCanHide()}
      onChange={column.getToggleVisibilityHandler()}
      type="checkbox"
    />
    {column.columnDef.header}
  </label>
))}
```

Use `row.getVisibleCells()` (not `row.getAllCells()`) when rendering.

---

## Column Ordering

> API: https://tanstack.com/table/latest/docs/api/features/column-ordering

### State

```ts
const [columnOrder, setColumnOrder] = useState<string[]>(['col1', 'col2', 'col3'])

state: { columnOrder },
onColumnOrderChange: setColumnOrder,
```

Order is affected by: Column Pinning → Manual Column Ordering → Grouping.

### DnD Recommendations (React)

1. Use `@dnd-kit/core` (recommended)
2. Avoid `react-dnd` with React 18+
3. Consider native browser drag events for lightweight solutions

---

## Column Pinning

> API: https://tanstack.com/table/latest/docs/api/features/column-pinning

### State

```ts
const [columnPinning, setColumnPinning] = useState<ColumnPinningState>({
  left: ['expand-column'],
  right: ['actions-column'],
})
```

### Key APIs

- `column.pin('left' | 'right' | false)` - Pin/unpin column
- `column.getIsPinned()` - Returns `'left'` | `'right'` | `false`
- `column.getStart()` / `column.getAfter()` - CSS position values for sticky pinning
- `column.getIsLastColumn('left')` / `column.getIsFirstColumn('right')` - For box-shadow

### Split Table vs Sticky CSS

- **Sticky CSS**: Use `table.getHeaderGroups()` + `row.getVisibleCells()` normally
- **Split tables**: Use `table.getLeftHeaderGroups()`, `table.getCenterHeaderGroups()`, `table.getRightHeaderGroups()` and corresponding row cell APIs

---

## Column Sizing & Resizing

> API: https://tanstack.com/table/latest/docs/api/features/column-sizing

### Defaults

```ts
{ size: 150, minSize: 20, maxSize: Number.MAX_SAFE_INTEGER }
```

Override per-column or with `defaultColumn` table option.

### Resize Mode

- `"onEnd"` (default) - Size updates after drag ends (better React performance)
- `"onChange"` - Immediate size updates during drag

```tsx
const table = useReactTable({
  columnResizeMode: 'onChange',
  columnResizeDirection: 'ltr', // or 'rtl'
})
```

### Resize Handler

```tsx
<div
  onMouseDown={header.getResizeHandler()}
  onTouchStart={header.getResizeHandler()}
/>
```

### Performance Tips

1. Calculate all column widths once upfront, memoized
2. Memoize table body while resizing
3. Use CSS variables for column widths

---

## Expanding

> API: https://tanstack.com/table/latest/docs/api/features/expanding

### Sub-Rows

```tsx
const table = useReactTable({
  data, columns,
  getSubRows: row => row.children,
  getCoreRowModel: getCoreRowModel(),
  getExpandedRowModel: getExpandedRowModel(),
})
```

### Custom Expanded UI (Detail Panels)

```tsx
const table = useReactTable({
  getRowCanExpand: row => true,
  getCoreRowModel: getCoreRowModel(),
  getExpandedRowModel: getExpandedRowModel(),
})

// In render:
{row.getIsExpanded() && (
  <tr>
    <td colSpan={row.getAllCells().length}>
      {/* Custom detail panel */}
    </td>
  </tr>
)}
```

### State

```ts
type ExpandedState = true | Record<string, boolean>
// true = all expanded, Record = specific rows

const [expanded, setExpanded] = useState<ExpandedState>({})
state: { expanded },
onExpandedChange: setExpanded,
```

### Toggle UI

```tsx
<button onClick={row.getToggleExpandedHandler()}>
  {row.getIsExpanded() ? '👇' : '👉'}
</button>
```

### Options

- `paginateExpandedRows` (table) - Default true. Set false to keep expanded rows on parent's page.
- `filterFromLeafRows` / `maxLeafRowFilterDepth` - Control filtering of expanded rows.

---

## Grouping

> API: https://tanstack.com/table/latest/docs/api/features/grouping

### Setup

```tsx
import { getGroupedRowModel, getExpandedRowModel } from '@tanstack/react-table'

const table = useReactTable({
  data, columns,
  getCoreRowModel: getCoreRowModel(),
  getGroupedRowModel: getGroupedRowModel(),
  getExpandedRowModel: getExpandedRowModel(),
})
```

### State

```ts
table.setGrouping(['column1', 'column2'])
```

### Aggregation

```ts
columnHelper.accessor('amount', {
  aggregationFn: 'sum', // built-in: sum, count, min, max, extent, mean, median, unique, uniqueCount
})
```

Custom:
```ts
aggregationFns: {
  myAgg: (columnId, leafRows, childRows) => { return aggregatedValue },
}
```

### Options

- `groupedColumnMode`: `'reorder'` | `'remove'` | `false`
- `manualGrouping`: `true` for server-side grouping

---

## Column Faceting

> API: https://tanstack.com/table/latest/docs/api/features/column-faceting

### Setup

```tsx
const table = useReactTable({
  getCoreRowModel: getCoreRowModel(),
  getFacetedRowModel: getFacetedRowModel(),
  getFacetedUniqueValues: getFacetedUniqueValues(),
  getFacetedMinMaxValues: getFacetedMinMaxValues(),
})
```

### Usage

```ts
// Unique values for autocomplete
const suggestions = Array.from(column.getFacetedUniqueValues().keys()).sort().slice(0, 5000)

// Min/max for range slider
const [min, max] = column.getFacetedMinMaxValues() ?? [0, 1]
```

---

## Row Pinning

> API: https://tanstack.com/table/latest/docs/api/features/row-pinning

Rows are split into top, center (unpinned), and bottom pinned rows.

---

## Virtualization

TanStack Table does not include virtualization, but integrates with [TanStack Virtual](https://tanstack.com/virtual/latest). See examples:

- [virtualized-columns](https://github.com/TanStack/table/tree/main/examples/react/virtualized-columns)
- [virtualized-rows](https://github.com/TanStack/table/tree/main/examples/react/virtualized-rows)
- [virtualized-infinite-scrolling](https://github.com/TanStack/table/tree/main/examples/react/virtualized-infinite-scrolling)

---

## Custom Features

> Example: https://tanstack.com/table/latest/docs/framework/react/examples/custom-features

Use `_features` table option to add custom features:

```ts
export const DensityFeature: TableFeature<any> = {
  getInitialState: (state) => ({ density: 'md', ...state }),
  getDefaultOptions: (table) => ({
    enableDensity: true,
    onDensityChange: makeStateUpdater('density', table),
  }),
  createTable: (table) => {
    table.setDensity = updater => { /* ... */ }
    table.toggleDensity = value => { /* ... */ }
  },
}

// Use declaration merging for TypeScript:
declare module '@tanstack/react-table' {
  interface TableState extends DensityTableState {}
  interface TableOptionsResolved<TData extends RowData> extends DensityOptions {}
  interface Table<TData extends RowData> extends DensityInstance {}
}

// Pass to table:
const table = useReactTable({ _features: [DensityFeature], ... })
```
