# TanStack Form - SSR & Meta-Framework Integration

## Table of Contents
- [TanStack Start](#tanstack-start)
- [Next.js App Router](#nextjs-app-router)
- [Remix](#remix)

## TanStack Start

### Setup

```bash
npm install @tanstack/react-form @tanstack/react-form-start
```

### Shared Form Options

```typescript
import { formOptions } from '@tanstack/react-form-start'

export const formOpts = formOptions({
  defaultValues: { firstName: '', age: 0 },
})
```

### Server Validation

```typescript
import { createServerValidate, ServerValidateError } from '@tanstack/react-form-start'

const serverValidate = createServerValidate({
  ...formOpts,
  onServerValidate: ({ value }) => {
    if (value.age < 12) return 'Server validation: You must be at least 12 to sign up'
  },
})

export const handleForm = createServerFn({ method: 'POST' })
  .inputValidator((data: unknown) => {
    if (!(data instanceof FormData)) throw new Error('Invalid form data')
    return data
  })
  .handler(async (ctx) => {
    try {
      const validatedData = await serverValidate(ctx.data)
      // Persist to database
    } catch (e) {
      if (e instanceof ServerValidateError) return e.response
      console.error(e)
      setResponseStatus(500)
      return 'There was an internal error'
    }
    return 'Form has validated successfully'
  })
```

### Get Form Data Server Function

```typescript
import { getFormData } from '@tanstack/react-form-start'

export const getFormDataFromServer = createServerFn({ method: 'GET' }).handler(
  async () => getFormData()
)
```

### Client Component

```tsx
import { mergeForm, useForm, useStore, useTransform } from '@tanstack/react-form-start'

export const Route = createFileRoute('/')({
  component: Home,
  loader: async () => ({ state: await getFormDataFromServer() }),
})

function Home() {
  const { state } = Route.useLoaderData()
  const form = useForm({
    ...formOpts,
    transform: useTransform((baseForm) => mergeForm(baseForm, state), [state]),
  })
  const formErrors = useStore(form.store, (formState) => formState.errors)

  return (
    <form action={handleForm.url} method="post" encType="multipart/form-data">
      {formErrors.map((error) => <p key={error as string}>{error}</p>)}
      <form.Field name="age" validators={{
        onChange: ({ value }) => value < 8 ? 'Client validation: You must be at least 8' : undefined,
      }}>
        {(field) => (
          <div>
            <input name={field.name} type="number" value={field.state.value}
              onChange={(e) => field.handleChange(e.target.valueAsNumber)} />
            {field.state.meta.errors.map((error) => <p key={error as string}>{error}</p>)}
          </div>
        )}
      </form.Field>
      <form.Subscribe selector={(formState) => [formState.canSubmit, formState.isSubmitting]}>
        {([canSubmit, isSubmitting]) => (
          <button type="submit" disabled={!canSubmit}>{isSubmitting ? '...' : 'Submit'}</button>
        )}
      </form.Subscribe>
    </form>
  )
}
```

## Next.js App Router

### Setup

```bash
npm install @tanstack/react-form @tanstack/react-form-nextjs
```

**Important**: Import from `@tanstack/react-form-nextjs` (not `@tanstack/react-form`) to avoid "useState in server component" errors.

### Shared Form Options

```typescript
// shared-code.ts
import { formOptions } from '@tanstack/react-form-nextjs'

export const formOpts = formOptions({
  defaultValues: { firstName: '', age: 0 },
})
```

### Server Action

```typescript
// action.ts
'use server'
import { ServerValidateError, createServerValidate } from '@tanstack/react-form-nextjs'
import { formOpts } from './shared-code'

const serverValidate = createServerValidate({
  ...formOpts,
  onServerValidate: ({ value }) => {
    if (value.age < 12) return 'Server validation: You must be at least 12 to sign up'
  },
})

export default async function someAction(prev: unknown, formData: FormData) {
  try {
    const validatedData = await serverValidate(formData)
    // Persist to database
  } catch (e) {
    if (e instanceof ServerValidateError) return e.formState
    throw e
  }
}
```

### Client Component

```tsx
// client-component.tsx
'use client'
import { useActionState } from 'react'
import { initialFormState, mergeForm, useForm, useStore, useTransform } from '@tanstack/react-form-nextjs'
import someAction from './action'
import { formOpts } from './shared-code'

export const ClientComp = () => {
  const [state, action] = useActionState(someAction, initialFormState)
  const form = useForm({
    ...formOpts,
    transform: useTransform((baseForm) => mergeForm(baseForm, state!), [state]),
  })
  const formErrors = useStore(form.store, (formState) => formState.errors)

  return (
    <form action={action as never} onSubmit={() => form.handleSubmit()}>
      {formErrors.map((error) => <p key={error as string}>{error}</p>)}
      <form.Field name="age" validators={{
        onChange: ({ value }) => value < 8 ? 'Client validation: You must be at least 8' : undefined,
      }}>
        {(field) => (
          <div>
            <input name={field.name} type="number" value={field.state.value}
              onChange={(e) => field.handleChange(e.target.valueAsNumber)} />
            {field.state.meta.errors.map((error) => <p key={error as string}>{error}</p>)}
          </div>
        )}
      </form.Field>
      <form.Subscribe selector={(formState) => [formState.canSubmit, formState.isSubmitting]}>
        {([canSubmit, isSubmitting]) => (
          <button type="submit" disabled={!canSubmit}>{isSubmitting ? '...' : 'Submit'}</button>
        )}
      </form.Subscribe>
    </form>
  )
}
```

## Remix

### Setup

```bash
npm install @tanstack/react-form @tanstack/react-form-remix
```

### Shared Form Options + Action

```typescript
// routes/_index/route.tsx
import { formOptions, createServerValidate, ServerValidateError } from '@tanstack/react-form-remix'
import type { ActionFunctionArgs } from '@remix-run/node'

export const formOpts = formOptions({
  defaultValues: { firstName: '', age: 0 },
})

const serverValidate = createServerValidate({
  ...formOpts,
  onServerValidate: ({ value }) => {
    if (value.age < 12) return 'Server validation: You must be at least 12 to sign up'
  },
})

export async function action({ request }: ActionFunctionArgs) {
  const formData = await request.formData()
  try {
    const validatedData = await serverValidate(formData)
    // Persist to database
  } catch (e) {
    if (e instanceof ServerValidateError) return e.formState
    throw e
  }
}
```

### Client Component

```tsx
import { Form, mergeForm, useActionData, useForm, useStore, useTransform } from '@tanstack/react-form'
import { initialFormState } from '@tanstack/react-form-remix'

export default function Index() {
  const actionData = useActionData<typeof action>()
  const form = useForm({
    ...formOpts,
    transform: useTransform(
      (baseForm) => mergeForm(baseForm, actionData ?? initialFormState),
      [actionData],
    ),
  })
  const formErrors = useStore(form.store, (formState) => formState.errors)

  return (
    <Form method="post" onSubmit={() => form.handleSubmit()}>
      {formErrors.map((error) => <p key={error as string}>{error}</p>)}
      <form.Field name="age" validators={{
        onChange: ({ value }) => value < 8 ? 'Must be at least 8' : undefined,
      }}>
        {(field) => (
          <div>
            <input name="age" type="number" value={field.state.value}
              onChange={(e) => field.handleChange(e.target.valueAsNumber)} />
            {field.state.meta.errors.map((error) => <p key={error as string}>{error}</p>)}
          </div>
        )}
      </form.Field>
      <form.Subscribe selector={(formState) => [formState.canSubmit, formState.isSubmitting]}>
        {([canSubmit, isSubmitting]) => (
          <button type="submit" disabled={!canSubmit}>{isSubmitting ? '...' : 'Submit'}</button>
        )}
      </form.Subscribe>
    </Form>
  )
}
```

### Key SSR Pattern

All meta-framework integrations share this pattern:
1. Define `formOptions` with shared form config
2. Create server-side validation with `createServerValidate`
3. Handle `ServerValidateError` in server action/handler
4. Client uses `useTransform` + `mergeForm` to sync server state
5. Both client and server validations run independently
