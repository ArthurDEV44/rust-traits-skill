# TanStack Form - Advanced Patterns

## Table of Contents
- [Array Fields](#array-fields)
- [Linked Fields](#linked-fields)
- [Listeners (Side Effects)](#listeners)
- [Reactivity (useStore & Subscribe)](#reactivity)
- [Async Initial Values](#async-initial-values)
- [Submission Handling & Meta](#submission-handling)
- [Focus Management](#focus-management)
- [UI Library Integration](#ui-library-integration)
- [React Native](#react-native)

## Array Fields

Use `mode="array"` on `form.Field`:

```tsx
<form.Field name="people" mode="array">
  {(field) => (
    <div>
      {field.state.value.map((_, i) => (
        <form.Field key={i} name={`people[${i}].name`}>
          {(subField) => (
            <div>
              <label>Name for person {i}</label>
              <input
                value={subField.state.value}
                onChange={(e) => subField.handleChange(e.target.value)}
              />
              <button type="button" onClick={() => field.removeValue(i)}>X</button>
            </div>
          )}
        </form.Field>
      ))}
      <button type="button" onClick={() => field.pushValue({ name: '', age: 0 })}>
        Add person
      </button>
    </div>
  )}
</form.Field>
```

Array methods: `pushValue`, `removeValue`, `swapValues`, `moveValue`, `insertValue`, `replaceValue`, `clearValues`.

## Linked Fields

Re-validate one field when another changes using `onChangeListenTo` / `onBlurListenTo`:

```tsx
<form.Field name="password">
  {(field) => (
    <input value={field.state.value} onChange={(e) => field.handleChange(e.target.value)} />
  )}
</form.Field>

<form.Field
  name="confirm_password"
  validators={{
    onChangeListenTo: ['password'],
    onChange: ({ value, fieldApi }) => {
      if (value !== fieldApi.form.getFieldValue('password')) return 'Passwords do not match'
      return undefined
    },
  }}
>
  {(field) => (
    <div>
      <input value={field.state.value} onChange={(e) => field.handleChange(e.target.value)} />
      {field.state.meta.errors.map((err) => <div key={err}>{err}</div>)}
    </div>
  )}
</form.Field>
```

## Listeners

React to triggers with side effects. Events: `onChange`, `onBlur`, `onMount`, `onSubmit`.

Field-level listeners:
```tsx
<form.Field
  name="country"
  listeners={{
    onChange: ({ value }) => {
      console.log(`Country changed to: ${value}, resetting province`)
      form.setFieldValue('province', '')
    },
  }}
>
  {(field) => (
    <input value={field.state.value} onChange={(e) => field.handleChange(e.target.value)} />
  )}
</form.Field>
```

Debounced listeners:
```tsx
listeners={{
  onChangeDebounceMs: 500,
  onChange: ({ value }) => { /* debounced handler */ },
}}
```

Form-level listeners (propagated to children for onChange/onBlur):
```tsx
const form = useForm({
  listeners: {
    onMount: ({ formApi }) => loggingService('mount', formApi.state.values),
    onChange: ({ formApi, fieldApi }) => {
      if (formApi.state.isValid) formApi.handleSubmit()
      console.log(fieldApi.name, fieldApi.state.value)
    },
    onChangeDebounceMs: 500,
  },
})
```

## Reactivity

### useStore

For reactive values in component logic. Always provide a selector:

```tsx
const firstName = useStore(form.store, (state) => state.values.firstName)
const errors = useStore(form.store, (state) => state.errorMap)
// BAD: const store = useStore(form.store) // unnecessary re-renders
```

### form.Subscribe

For reactive UI without component-level re-renders:

```tsx
<form.Subscribe
  selector={(state) => state.values.firstName}
  children={(firstName) => <div>Hello {firstName}</div>}
/>
```

Choose `useStore` for logic, `form.Subscribe` for UI optimization.

## Async Initial Values

Combine with TanStack Query:

```tsx
import { useForm } from '@tanstack/react-form'
import { useQuery } from '@tanstack/react-query'

export default function App() {
  const { data, isLoading } = useQuery({
    queryKey: ['data'],
    queryFn: async () => {
      await new Promise((resolve) => setTimeout(resolve, 1000))
      return { firstName: 'FirstName', lastName: 'LastName' }
    },
  })

  const form = useForm({
    defaultValues: {
      firstName: data?.firstName ?? '',
      lastName: data?.lastName ?? '',
    },
    onSubmit: async ({ value }) => console.log(value),
  })

  if (isLoading) return <p>Loading...</p>
  return /* form JSX */
}
```

## Submission Handling

### Submit Meta

Pass additional data to submission handler:

```tsx
type FormMeta = { submitAction: 'continue' | 'backToMenu' | null }

const form = useForm({
  defaultValues: { data: '' },
  onSubmitMeta: { submitAction: null } as FormMeta,
  onSubmit: async ({ value, meta }) => {
    console.log(`Selected action - ${meta.submitAction}`, value)
  },
})

// In JSX:
<button type="submit" onClick={() => form.handleSubmit({ submitAction: 'continue' })}>
  Submit and continue
</button>
<button type="submit" onClick={() => form.handleSubmit({ submitAction: 'backToMenu' })}>
  Submit and back to menu
</button>
```

### Schema Transform in onSubmit

Standard Schema validation does NOT preserve transforms. Parse in `onSubmit`:

```tsx
const schema = z.object({ age: z.string().transform((age) => Number(age)) })

const form = useForm({
  defaultValues: { age: '13' } as z.input<typeof schema>,
  validators: { onChange: schema },
  onSubmit: ({ value }) => {
    const result = schema.parse(value) // result.age is number
  },
})
```

## Focus Management

### React DOM

```tsx
const form = useForm({
  defaultValues: { age: 0 },
  onSubmitInvalid() {
    const invalidInput = document.querySelector('[aria-invalid="true"]') as HTMLInputElement
    invalidInput?.focus()
  },
})

// Set aria-invalid on fields:
<input aria-invalid={!field.state.meta.isValid && field.state.meta.isTouched} />
```

### React Native

Manually track refs:

```tsx
const fields = useRef([] as Array<{ input: TextInput; name: string }>)

// In onSubmitInvalid:
const errorMap = formApi.state.errorMap.onChange
for (const input of fields.current) {
  if (!!errorMap[input.name]) { input.input.focus(); break }
}

// On field render:
<TextInput ref={(input) => { fields.current[0] = { input, name: field.name } }} />
```

## UI Library Integration

TanStack Form is headless. Works with any UI library. Key pattern:

```tsx
<form.Field name="fieldName" children={({ state, handleChange, handleBlur }) => (
  <UIComponent
    value={state.value}
    onChange={(e) => handleChange(e.target.value)}
    onBlur={handleBlur}
  />
)} />
```

Framework-specific notes:
- **shadcn/ui Checkbox**: Use `onCheckedChange` instead of `onChange`
- **Chakra UI Checkbox**: Composable parts (`Checkbox.Root`, `.Control`, `.Label`). Use `!!details.checked` to coerce indeterminate state
- **Material UI**: Standard `onChange`/`onBlur` pattern
- **Mantine**: Standard `onChange`/`onBlur` pattern

shadcn/ui has a dedicated TanStack Form guide: https://ui.shadcn.com/docs/forms/tanstack-form

## React Native

Works out of the box. Use `TextInput`, `onChangeText` instead of `onChange`:

```tsx
<form.Field name="age" validators={{ onChange: (val) => val < 13 ? 'Must be 13+' : undefined }}>
  {(field) => (
    <>
      <Text>Age:</Text>
      <TextInput value={field.state.value} onChangeText={field.handleChange} />
      {!field.state.meta.isValid && <Text>{field.state.meta.errors.join(', ')}</Text>}
    </>
  )}
</form.Field>
```
