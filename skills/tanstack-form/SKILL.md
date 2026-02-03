---
name: tanstack-form
description: "TanStack Form for React: type-safe, performant, composable form management with validation, arrays, linked fields, listeners, and SSR support. Use when writing, reviewing, or refactoring React form code with TanStack Form: (1) Creating forms with useForm or createFormHook, (2) Adding field-level or form-level validation (sync, async, schema-based with Zod/Valibot/ArkType), (3) Working with form.Field, form.AppField, form.Subscribe, useStore, (4) Managing array fields, linked fields, or dynamic validation, (5) Composing forms with withForm, withFieldGroup, or createFormHook, (6) Integrating with UI libraries (shadcn/ui, Material UI, Mantine, Chakra UI), (7) Setting up SSR with Next.js, TanStack Start, or Remix, (8) Debugging form state, TypeScript errors, or controlled input warnings, (9) Using listeners for side effects, (10) Handling form submission with meta or schema transforms."
---

# TanStack Form (React)

## Installation

```bash
npm install @tanstack/react-form
```

Meta-framework adapters: `@tanstack/react-form-start` | `@tanstack/react-form-nextjs` | `@tanstack/react-form-remix`

## Core Patterns

### Production Setup (Recommended)

Use `createFormHook` to bind reusable UI components and reduce boilerplate:

```tsx
import { createFormHookContexts, createFormHook } from '@tanstack/react-form'

const { fieldContext, formContext, useFieldContext, useFormContext } = createFormHookContexts()

const { useAppForm, withForm } = createFormHook({
  fieldComponents: { TextField, NumberField },
  formComponents: { SubmitButton },
  fieldContext,
  formContext,
})
```

Then use `form.AppField` (not `form.Field`) and `form.AppForm` for bound components.

### One-off Forms

```tsx
import { useForm } from '@tanstack/react-form'

const form = useForm({
  defaultValues: { name: '', age: 0 },
  onSubmit: ({ value }) => console.log(value),
})

// JSX: form.Field with name, validators, children render prop
```

### TypeScript: Never Pass Generics

Infer types from `defaultValues`. Use `useForm({ defaultValues: typedObj })` not `useForm<Type>()`.

## Validation

- **Timing**: `onChange`, `onBlur`, `onSubmit`, `onMount`, `onDynamic` (with `revalidateLogic()`)
- **Level**: Field (`validators` prop on Field) or Form (`validators` option on useForm)
- **Sync/Async**: `onChange` + `onChangeAsync`, with `asyncDebounceMs` for debounce
- **Schema**: Zod, Valibot, ArkType, Yup via Standard Schema spec
- **Errors**: `field.state.meta.errors` (array), `field.state.meta.errorMap` (by source)
- **Custom errors**: Return any truthy value (string, number, object, array)
- **Linked fields**: `onChangeListenTo: ['otherField']` re-validates when linked field changes

See [references/validation.md](references/validation.md) for complete validation patterns.

## Key APIs

| API | Purpose |
|-----|---------|
| `useForm` / `useAppForm` | Create form instance |
| `form.Field` / `form.AppField` | Render a field with render prop |
| `form.Subscribe` | Subscribe to form state (UI-level, no component re-render) |
| `useStore(form.store, selector)` | Subscribe to form state (component-level, requires selector) |
| `form.handleSubmit()` | Trigger submission |
| `form.reset()` | Reset form (use `e.preventDefault()` with `type="reset"`) |
| `formOptions()` | Shared form config |
| `withForm` | Break big forms into typed child components |
| `withFieldGroup` | Reuse field groups across forms |

## Array Fields

Use `mode="array"` on Field. Methods: `pushValue`, `removeValue`, `swapValues`, `moveValue`, `insertValue`, `replaceValue`, `clearValues`. Access sub-fields via `name={\`items[${i}].prop\`}`.

## Listeners

Side effects on `onChange`, `onBlur`, `onMount`, `onSubmit` via `listeners` prop. Supports debounce with `onChangeDebounceMs`. Available at field and form level.

## References

Detailed guides with full code examples:

- **[Quick Start & Basics](references/quick-start.md)**: Installation, philosophy, TypeScript, basic concepts, field state, devtools setup
- **[Validation](references/validation.md)**: All validation patterns (timing, sync/async, schema, dynamic, custom errors, disableErrorFlat)
- **[Form Composition](references/form-composition.md)**: createFormHook, pre-bound components, withForm, withFieldGroup, tree-shaking, useTypedAppFormContext
- **[Advanced Patterns](references/advanced-patterns.md)**: Arrays, linked fields, listeners, reactivity, async initial values, submission handling/meta, focus management, UI library integration, React Native
- **[SSR & Meta-Frameworks](references/ssr-meta-frameworks.md)**: TanStack Start, Next.js App Router, Remix integration with server validation

## Official Documentation

- https://tanstack.com/form/latest
- https://tanstack.com/form/latest/docs/framework/react/quick-start
- https://tanstack.com/form/latest/docs/framework/react/guides/basic-concepts
