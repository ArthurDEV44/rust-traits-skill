# TanStack Form - Validation

## Table of Contents
- [Validation Timing](#validation-timing)
- [Displaying Errors](#displaying-errors)
- [Field-level vs Form-level Validation](#field-level-vs-form-level-validation)
- [Setting Field Errors from Form Validators](#setting-field-errors-from-form-validators)
- [Async Validation](#async-validation)
- [Debouncing](#debouncing)
- [Schema Libraries (Standard Schema)](#schema-libraries)
- [Dynamic Validation (onDynamic)](#dynamic-validation)
- [Custom Error Types](#custom-error-types)
- [disableErrorFlat](#disableerrorflat)
- [Preventing Invalid Submissions](#preventing-invalid-submissions)

## Validation Timing

Control when validation runs via validator keys: `onChange`, `onBlur`, `onSubmit`, `onMount`.

```tsx
<form.Field
  name="age"
  validators={{
    onChange: ({ value }) => value < 13 ? 'You must be 13 to make an account' : undefined,
  }}
>
  {(field) => (
    <>
      <label htmlFor={field.name}>Age:</label>
      <input
        id={field.name}
        name={field.name}
        value={field.state.value}
        type="number"
        onChange={(e) => field.handleChange(e.target.valueAsNumber)}
      />
      {!field.state.meta.isValid && <em role="alert">{field.state.meta.errors.join(', ')}</em>}
    </>
  )}
</form.Field>
```

For onBlur, add `onBlur={field.handleBlur}` to input and use `onBlur` validator key.

Combine multiple timings on the same field:

```tsx
validators={{
  onChange: ({ value }) => value < 13 ? 'You must be 13' : undefined,
  onBlur: ({ value }) => value < 0 ? 'Invalid value' : undefined,
}}
```

## Displaying Errors

Array of all errors:
```tsx
{!field.state.meta.isValid && <em>{field.state.meta.errors.join(',')}</em>}
```

Error by source via `errorMap`:
```tsx
{field.state.meta.errorMap['onChange'] ? <em>{field.state.meta.errorMap['onChange']}</em> : null}
```

Errors are fully typed based on validator return types.

## Field-level vs Form-level Validation

Form-level validators in `useForm`:

```tsx
const form = useForm({
  defaultValues: { age: 0 },
  validators: {
    onChange({ value }) {
      if (value.age < 13) return 'Must be 13 or older to sign'
      return undefined
    },
  },
})

// Display form-level errors
const formErrorMap = useStore(form.store, (state) => state.errorMap)
// or
<form.Subscribe selector={(state) => [state.errorMap]} children={([errorMap]) => /* ... */} />
```

**Note**: Field-specific validation overrides form-level validation for the same field.

## Setting Field Errors from Form Validators

Return `{ form?: string, fields: Record<string, string> }` from form validators:

```tsx
validators: {
  onSubmitAsync: async ({ value }) => {
    const hasErrors = await verifyDataOnServer(value)
    if (hasErrors) {
      return {
        form: 'Invalid data',
        fields: {
          age: 'Must be 13 or older to sign',
          'socials[0].url': 'The provided URL does not exist',
          'details.email': 'An email is required',
        },
      }
    }
    return null
  },
}
```

## Async Validation

Use `onChangeAsync`, `onBlurAsync`, etc.:

```tsx
validators={{
  onChangeAsync: async ({ value }) => {
    await new Promise((resolve) => setTimeout(resolve, 1000))
    return value < 13 ? 'You must be 13 to make an account' : undefined
  },
}}
```

Sync and async can coexist. Sync runs first; async runs only if sync passes (unless `asyncAlways: true`).

## Debouncing

```tsx
<form.Field
  name="age"
  asyncDebounceMs={500}
  validators={{
    onChangeAsyncDebounceMs: 1500, // override per-validator
    onChangeAsync: async ({ value }) => { /* ... */ },
    onBlurAsync: async ({ value }) => { /* ... */ }, // uses 500ms default
  }}
/>
```

## Schema Libraries

Supports Standard Schema spec: Zod (v3.24+), Valibot (v1.0+), ArkType (v2.1.20+), Yup (v1.7+).

Form-level schema:
```tsx
const form = useForm({
  defaultValues: { age: 0 },
  validators: { onChange: z.object({ age: z.number().gte(13, 'Must be 13+') }) },
})
```

Field-level schema:
```tsx
validators={{
  onChange: z.number().gte(13, 'Must be 13+'),
  onChangeAsyncDebounceMs: 500,
  onChangeAsync: z.number().refine(async (value) => {
    const currentAge = await fetchCurrentAgeOnProfile()
    return value >= currentAge
  }, { message: 'You can only increase the age' }),
}}
```

Combine schema with custom logic via `fieldApi.parseValueWithSchema()`:
```tsx
validators={{
  onChangeAsync: async ({ value, fieldApi }) => {
    const errors = fieldApi.parseValueWithSchema(z.number().gte(13, 'Must be 13+'))
    if (errors) return errors
    // continue with custom validation
  },
}}
```

**Note**: Schema validation does NOT provide transformed values. Parse in `onSubmit` for transforms.

## Dynamic Validation

Use `onDynamic` with `revalidateLogic()` to change validation rules based on form submission state:

```tsx
import { revalidateLogic, useForm } from '@tanstack/react-form'

const form = useForm({
  defaultValues: { firstName: '', lastName: '' },
  validationLogic: revalidateLogic(), // Required! Default: mode='submit', modeAfterSubmission='change'
  validators: {
    onDynamic: ({ value }) => {
      if (!value.firstName) return { firstName: 'A first name is required' }
      return undefined
    },
  },
})
```

`revalidateLogic` options:
- `mode`: `'change'` | `'blur'` | `'submit'` (default) - before first submission
- `modeAfterSubmission`: `'change'` (default) | `'blur'` | `'submit'` - after first submission

Access errors: `form.state.errorMap.onDynamic?.firstName`

Works with fields, async (`onDynamicAsync`, `onDynamicAsyncDebounceMs`), and Standard Schemas.

## Custom Error Types

Return any truthy value as error: strings, numbers, booleans, objects, arrays.

Object errors:
```tsx
validators={{
  onChange: ({ value }) => {
    if (!value.includes('@')) {
      return { message: 'Invalid email format', severity: 'error', code: 1001 }
    }
    return undefined
  },
}}
```

Array errors (multiple messages per field):
```tsx
validators={{
  onChange: ({ value }) => {
    const errors = []
    if (value.length < 8) errors.push('Password too short')
    if (!/[A-Z]/.test(value)) errors.push('Missing uppercase letter')
    if (!/[0-9]/.test(value)) errors.push('Missing number')
    return errors.length ? errors : undefined
  },
}}
```

## disableErrorFlat

Preserves error sources instead of flattening into single `errors` array:

```tsx
<form.Field name="email" disableErrorFlat
  validators={{
    onChange: ({ value }) => !value.includes('@') ? 'Invalid email' : undefined,
    onBlur: ({ value }) => !value.endsWith('.com') ? 'Only .com domains' : undefined,
  }}
/>
// Access: field.state.meta.errorMap.onChange, field.state.meta.errorMap.onBlur
```

## Preventing Invalid Submissions

```tsx
<form.Subscribe
  selector={(state) => [state.canSubmit, state.isSubmitting]}
  children={([canSubmit, isSubmitting]) => (
    <button type="submit" disabled={!canSubmit}>{isSubmitting ? '...' : 'Submit'}</button>
  )}
/>
```

Combine `canSubmit` with `isPristine` to prevent submission before interaction: `!canSubmit || isPristine`.
