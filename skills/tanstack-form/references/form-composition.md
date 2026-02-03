# TanStack Form - Form Composition

## Table of Contents
- [createFormHook](#createformhook)
- [Pre-bound Field Components](#pre-bound-field-components)
- [Pre-bound Form Components](#pre-bound-form-components)
- [withForm (Breaking Big Forms)](#withform)
- [withFieldGroup (Reusable Field Groups)](#withfieldgroup)
- [Context Fallback (useTypedAppFormContext)](#context-fallback)
- [Tree-shaking with React.lazy](#tree-shaking)
- [Full Example](#full-example)
- [API Usage Chart](#api-usage-chart)

## createFormHook

Create custom form hooks with pre-bound UI components:

```tsx
import { createFormHookContexts, createFormHook } from '@tanstack/react-form'

export const { fieldContext, formContext, useFieldContext, useFormContext } =
  createFormHookContexts()

const { useAppForm, withForm } = createFormHook({
  fieldContext,
  formContext,
  fieldComponents: {},
  formComponents: {},
})
```

## Pre-bound Field Components

Create field components using `useFieldContext`:

```tsx
import { useFieldContext } from './form-context.tsx'

export function TextField({ label }: { label: string }) {
  const field = useFieldContext<string>()
  return (
    <label>
      <span>{label}</span>
      <input value={field.state.value} onChange={(e) => field.handleChange(e.target.value)} />
    </label>
  )
}
```

Register and use:

```tsx
const { useAppForm } = createFormHook({
  fieldContext, formContext,
  fieldComponents: { TextField },
  formComponents: {},
})

// Usage - AppField instead of Field
<form.AppField name="firstName" children={(field) => <field.TextField label="First Name" />} />
```

**Performance note**: Context values are static class instances with reactive properties (via TanStack Store), NOT reactive context values. No unnecessary re-renders.

## Pre-bound Form Components

For shared form-level components like submit buttons:

```tsx
function SubscribeButton({ label }: { label: string }) {
  const form = useFormContext()
  return (
    <form.Subscribe selector={(state) => state.isSubmitting}>
      {(isSubmitting) => <button type="submit" disabled={isSubmitting}>{label}</button>}
    </form.Subscribe>
  )
}

const { useAppForm } = createFormHook({
  fieldContext, formContext,
  fieldComponents: { TextField },
  formComponents: { SubscribeButton },
})

// Usage - wrap in AppForm
<form.AppForm>
  <form.SubscribeButton label="Submit" />
</form.AppForm>
```

## withForm

Break large forms into smaller components with full type safety:

```tsx
const ChildForm = withForm({
  defaultValues: { firstName: 'John', lastName: 'Doe' }, // Type-checking only, not runtime
  props: { title: 'Child Form' }, // Optional extra props with defaults
  render: function Render({ form, title }) {
    return (
      <div>
        <p>{title}</p>
        <form.AppField name="firstName" children={(field) => <field.TextField label="First Name" />} />
        <form.AppForm><form.SubscribeButton label="Submit" /></form.AppForm>
      </div>
    )
  },
})

// Usage
<ChildForm form={form} title={'Testing'} />
```

**Important**: Use named function `render: function Render({ ... })` to avoid ESLint hook errors.

Can spread `formOptions` instead of redeclaring: `withForm({ ...formOpts, render: ... })`

## withFieldGroup

Reuse field groups across multiple forms:

```tsx
type PasswordFields = { password: string; confirm_password: string }

const FieldGroupPasswordFields = withFieldGroup({
  defaultValues: { password: '', confirm_password: '' } as PasswordFields,
  props: { title: 'Password' },
  render: function Render({ group, title }) {
    return (
      <div>
        <h2>{title}</h2>
        <group.AppField name="password">
          {(field) => <field.TextField label="Password" />}
        </group.AppField>
        <group.AppField
          name="confirm_password"
          validators={{
            onChangeListenTo: ['password'],
            onChange: ({ value, fieldApi }) => {
              if (value !== group.getFieldValue('password')) return 'Passwords do not match'
              return undefined
            },
          }}
        >
          {(field) => <field.TextField label="Confirm Password" />}
        </group.AppField>
      </div>
    )
  },
})
```

Usage in forms - `fields` prop specifies location:

```tsx
// Nested fields
<FieldGroupPasswordFields form={form} fields="account_data" title="Passwords" />

// In array
<form.Field name="linked_accounts" mode="array">
  {(field) => field.state.value.map((account, i) => (
    <FieldGroupPasswordFields key={account.provider} form={form} fields={`linked_accounts[${i}]`} title={account.provider} />
  ))}
</form.Field>

// Field mapping (top-level or renamed)
<FieldGroupPasswordFields form={form} fields={{ password: 'password', confirm_password: 'confirm_password' }} />
```

`createFieldMap(defaultValues)` helper generates identity mapping for top-level fields.

`group` object provides: `AppField`, `AppForm`, `Field`, `Subscribe`, `store`, `getFieldValue`, `form` (parent form).

## Context Fallback

For edge cases where passing `form` prop is not feasible (e.g., TanStack Router `<Outlet />`):

```tsx
const { useAppForm, useTypedAppFormContext } = createFormHook({ /* ... */ })

function ParentComponent() {
  const form = useAppForm({ ...formOptions })
  return <form.AppForm><ChildComponent /></form.AppForm>
}

function ChildComponent() {
  const form = useTypedAppFormContext({ ...formOptions })
  // Has access to form components, field components, and fields
}
```

**Warning**: No type safety guarantee. Prefer `withForm` whenever possible.

## Tree-shaking

Use `React.lazy` + `Suspense` for large component sets:

```tsx
import { lazy } from 'react'
const TextField = lazy(() => import('../components/text-field.tsx'))

const { useAppForm } = createFormHook({
  fieldContext, formContext,
  fieldComponents: { TextField },
  formComponents: {},
})

// Wrap in Suspense
<Suspense fallback={<p>Loading...</p>}><PeoplePage /></Suspense>
```

## Full Example

```tsx
// /src/hooks/form.ts
const { fieldContext, useFieldContext, formContext, useFormContext } = createFormHookContexts()

function TextField({ label }: { label: string }) {
  const field = useFieldContext<string>()
  return <label><span>{label}</span><input value={field.state.value} onChange={(e) => field.handleChange(e.target.value)} /></label>
}

function SubscribeButton({ label }: { label: string }) {
  const form = useFormContext()
  return <form.Subscribe selector={(state) => state.isSubmitting}>{(isSubmitting) => <button disabled={isSubmitting}>{label}</button>}</form.Subscribe>
}

const { useAppForm, withForm } = createFormHook({
  fieldComponents: { TextField },
  formComponents: { SubscribeButton },
  fieldContext, formContext,
})

// /src/features/people/shared-form.ts
const formOpts = formOptions({ defaultValues: { firstName: 'John', lastName: 'Doe' } })

// /src/features/people/nested-form.ts
const ChildForm = withForm({
  ...formOpts,
  props: { title: 'Child Form' },
  render: function Render({ form, title }) {
    return (
      <div>
        <p>{title}</p>
        <form.AppField name="firstName" children={(field) => <field.TextField label="First Name" />} />
        <form.AppForm><form.SubscribeButton label="Submit" /></form.AppForm>
      </div>
    )
  },
})

// /src/features/people/page.ts
const Parent = () => {
  const form = useAppForm({ ...formOpts })
  return <ChildForm form={form} title={'Testing'} />
}
```

## API Usage Chart

- **One-off forms**: `useForm` + `form.Field`
- **Production apps**: `createFormHook` + `useAppForm` + `form.AppField` / `form.AppForm`
- **Large forms split into sections**: `withForm`
- **Reusable field groups**: `withFieldGroup`
- **Edge cases (no prop drilling)**: `useTypedAppFormContext`
