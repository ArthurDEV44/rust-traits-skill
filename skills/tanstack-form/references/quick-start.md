# TanStack Form - Quick Start & Basic Concepts

## Table of Contents
- [Installation](#installation)
- [Philosophy](#philosophy)
- [TypeScript Guidelines](#typescript-guidelines)
- [Quick Start with createFormHook](#quick-start-with-createformhook)
- [Quick Start with useForm](#quick-start-with-useform)
- [Basic Concepts](#basic-concepts)
- [Reset Buttons](#reset-buttons)
- [Debugging](#debugging)
- [Devtools](#devtools)

## Installation

```bash
npm install @tanstack/react-form
```

Meta-framework adapters:
- TanStack Start: `@tanstack/react-form-start`
- Next.js: `@tanstack/react-form-nextjs`
- Remix: `@tanstack/react-form-remix`

Devtools:
```bash
npm i @tanstack/react-devtools @tanstack/react-form-devtools
```

## Philosophy

Key principles:
- **Unified APIs**: One API surface, not multiple overlapping ones
- **Flexibility**: Multiple validation timings (blur, change, submit, mount), strategies (field/form), schema libs (Zod, Valibot, ArkType), async validation with debounce
- **Controlled**: Predictable state, easier testing, non-DOM support (React Native, Three.js), enhanced conditional logic
- **No generics needed**: Infer everything from `defaultValues` runtime defaults. Never `useForm<MyForm>()`, always `useForm({ defaultValues: myObj })`
- **Wrap into your design system**: Use `createFormHook` to export pre-bound components

## TypeScript Guidelines

Never pass generics. Use typed `defaultValues`:

```tsx
interface Person { name: string; age: number }
const defaultPerson: Person = { name: 'Bill Luo', age: 24 }

useForm({ defaultValues: defaultPerson })
```

## Quick Start with createFormHook

Recommended for production. Reduces boilerplate:

```tsx
import { createFormHook, createFormHookContexts } from '@tanstack/react-form'
import { TextField, NumberField, SubmitButton } from '~our-app/ui-library'
import { z } from 'zod'

const { fieldContext, formContext } = createFormHookContexts()

const { useAppForm } = createFormHook({
  fieldComponents: { TextField, NumberField },
  formComponents: { SubmitButton },
  fieldContext,
  formContext,
})

const PeoplePage = () => {
  const form = useAppForm({
    defaultValues: { username: '', age: 0 },
    validators: {
      onChange: z.object({
        username: z.string(),
        age: z.number().min(13),
      }),
    },
    onSubmit: ({ value }) => {
      alert(JSON.stringify(value, null, 2))
    },
  })

  return (
    <form onSubmit={(e) => { e.preventDefault(); form.handleSubmit() }}>
      <form.AppField name="username" children={(field) => <field.TextField label="Full Name" />} />
      <form.AppField name="age" children={(field) => <field.NumberField label="Age" />} />
      <form.AppForm><form.SubmitButton /></form.AppForm>
    </form>
  )
}
```

All `useForm` properties work in `useAppForm`. All `form.Field` properties work in `form.AppField`.

## Quick Start with useForm

For one-off forms or learning:

```tsx
import { useForm } from '@tanstack/react-form'

const form = useForm({
  defaultValues: { username: '', age: 0 },
  onSubmit: ({ value }) => { alert(JSON.stringify(value, null, 2)) },
})

// In JSX:
<form.Field
  name="age"
  validators={{
    onChange: ({ value }) => value > 13 ? undefined : 'Must be 13 or older',
  }}
  children={(field) => (
    <>
      <input
        name={field.name}
        value={field.state.value}
        onBlur={field.handleBlur}
        type="number"
        onChange={(e) => field.handleChange(e.target.valueAsNumber)}
      />
      {!field.state.meta.isValid && <em>{field.state.meta.errors.join(',')}</em>}
    </>
  )}
/>
```

## Basic Concepts

### formOptions

Share config between forms:

```tsx
const formOpts = formOptions({ defaultValues: { firstName: '', lastName: '' } })
const form = useForm({ ...formOpts, onSubmit: async ({ value }) => console.log(value) })
```

### Field Component

```tsx
<form.Field
  name="firstName"
  children={(field) => (
    <input
      value={field.state.value}
      onBlur={field.handleBlur}
      onChange={(e) => field.handleChange(e.target.value)}
    />
  )}
/>
```

ESLint config for `children` as prop:
```json
{ "rules": { "react/no-children-prop": [true, { "allowFunctions": true }] } }
```

### Field State

```tsx
const { value, meta: { errors, isValidating } } = field.state
const { isTouched, isDirty, isPristine, isBlurred, isDefaultValue } = field.state.meta
```

- **isTouched**: `true` once user changes or blurs
- **isDirty**: `true` once value changed (persistent, even if reverted). Opposite of `isPristine`
- **isPristine**: `true` until value changed. Opposite of `isDirty`
- **isBlurred**: `true` once field loses focus
- **isDefaultValue**: `true` when current value equals default value

Non-persistent dirty: `const nonPersistentIsDirty = !isDefaultValue`

### Reactivity

Subscribe to form state changes with `useStore` or `form.Subscribe`:

```tsx
// useStore - triggers component re-render
const firstName = useStore(form.store, (state) => state.values.firstName)

// form.Subscribe - only re-renders the Subscribe subtree
<form.Subscribe
  selector={(state) => [state.canSubmit, state.isSubmitting]}
  children={([canSubmit, isSubmitting]) => (
    <button type="submit" disabled={!canSubmit}>{isSubmitting ? '...' : 'Submit'}</button>
  )}
/>
```

Always provide a selector to `useStore` to avoid unnecessary re-renders. Do NOT use `useField` for reactivity.

## Reset Buttons

Prevent default HTML reset behavior:

```tsx
<button type="reset" onClick={(e) => { e.preventDefault(); form.reset() }}>Reset</button>
// OR
<button type="button" onClick={() => form.reset()}>Reset</button>
```

## Debugging

Common errors:
- **"Changing an uncontrolled input to be controlled"**: Missing `defaultValues` in `useForm`
- **Field value is `unknown`**: Form type too large. Cast with `field.state.value as string`
- **"Type instantiation is excessively deep"**: TypeScript edge case, report to GitHub

## Devtools

```tsx
import { TanStackDevtools } from '@tanstack/react-devtools'
import { formDevtoolsPlugin } from '@tanstack/react-form-devtools'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
    <TanStackDevtools plugins={[formDevtoolsPlugin()]} />
  </StrictMode>,
)
```
