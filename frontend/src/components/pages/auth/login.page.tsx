import { useNavigate } from '@tanstack/react-router'
import { LogInIcon } from 'lucide-react'
import { useState } from 'react'
import { useAppViewmodel } from '@/app.viewmodel'
import { LabeledInput } from '@/components/molecules/labeled-input'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from '@/components/ui/card'
import { AuthShell } from './auth-shell'

export const LoginPage = () => {
  const appViewmodel = useAppViewmodel()
  const navigate = useNavigate()
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [isSaving, setIsSaving] = useState(false)

  const handleSubmit = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    setIsSaving(true)
    setError(null)

    try {
      await appViewmodel.login({ email, password })
      await navigate({ to: '/' })
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : 'Unable to log in.')
    } finally {
      setIsSaving(false)
    }
  }

  return (
    <AuthShell>
      <Card>
        <CardHeader>
          <CardTitle>Log in</CardTitle>
        </CardHeader>
        <form onSubmit={handleSubmit}>
          <CardContent className="space-y-4">
            <LabeledInput autoComplete="email" label="Email" type="email" value={email} onChange={setEmail} />
            <LabeledInput
              autoComplete="current-password"
              label="Password"
              type="password"
              value={password}
              onChange={setPassword}
            />
            {error ? <p className="text-sm text-destructive">{error}</p> : null}
          </CardContent>
          <CardFooter className="justify-end">
            <Button disabled={!email.trim() || !password.trim() || isSaving} type="submit">
              <LogInIcon className="size-4" />
              {isSaving ? 'Logging in...' : 'Log in'}
            </Button>
          </CardFooter>
        </form>
      </Card>
    </AuthShell>
  )
}
