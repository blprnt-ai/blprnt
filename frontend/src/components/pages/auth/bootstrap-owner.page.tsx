import { ArrowRightIcon, UserIcon } from 'lucide-react'
import { observer } from 'mobx-react-lite'
import { useNavigate } from '@tanstack/react-router'
import { useEffect, useState } from 'react'
import { useAppViewmodel } from '@/app.viewmodel'
import { LabeledInput } from '@/components/molecules/labeled-input'
import { LabeledSelect } from '@/components/molecules/labeled-select'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from '@/components/ui/card'
import { ColoredSpan, colors, fallbackColor } from '@/components/ui/colors'
import { EmployeeLabel, employeeIcons, fallbackIcon } from '@/components/ui/employee-label'
import { employeesApi } from '@/lib/api/employees'
import { EmployeeModel } from '@/models/employee.model'
import { AuthShell } from './auth-shell'

export const BootstrapOwnerPage = observer(() => {
  const appViewmodel = useAppViewmodel()
  const navigate = useNavigate()
  const [owner] = useState(() => new EmployeeModel())
  const [error, setError] = useState<string | null>(null)
  const [isSaving, setIsSaving] = useState(false)
  const isLoginSetup = appViewmodel.hasOwner && !appViewmodel.isOwnerLoginConfigured

  useEffect(() => {
    if (!isLoginSetup) return

    let isMounted = true
    void employeesApi
      .getOwner()
      .then((existingOwner) => {
        if (!existingOwner || !isMounted) return
        owner.name = existingOwner.name
        owner.color = existingOwner.color as typeof owner.color
        owner.icon = existingOwner.icon
      })
      .catch(() => {})

    return () => {
      isMounted = false
    }
  }, [isLoginSetup, owner])

  const handleSubmit = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    setIsSaving(true)
    setError(null)

    try {
      await appViewmodel.bootstrapOwner(owner.toBootstrapOwnerPayload())
      await navigate({ to: '/' })
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : 'Unable to create owner login.')
    } finally {
      setIsSaving(false)
    }
  }

  return (
    <AuthShell>
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <UserIcon className="size-5" />
            {isLoginSetup ? 'Set up owner login' : 'Welcome to blprnt'}
          </CardTitle>
        </CardHeader>
        <form onSubmit={handleSubmit}>
          <CardContent className="space-y-4">
            <LabeledInput label="Name" placeholder="Beff Jezos" value={owner.name} onChange={(value) => (owner.name = value)} />
            <div className="grid grid-cols-2 gap-3">
              <LabeledSelect
                label="Color"
                value={owner.color}
                options={colors.map((color) => ({
                  label: (
                    <>
                      <ColoredSpan className="size-4 rounded-full" color={color.color} />
                      <span>{color.name}</span>
                    </>
                  ),
                  value: color.color,
                }))}
                selectedValue={
                  <>
                    <ColoredSpan className="size-4 rounded-full" color={owner.color} />
                    {owner.selectedColor.name}
                  </>
                }
                onChange={(value) => {
                  owner.color = value ?? fallbackColor
                }}
              />
              <LabeledSelect
                label="Icon"
                value={owner.icon}
                options={employeeIcons.map((icon) => ({
                  label: <EmployeeLabel color={owner.color} Icon={icon.icon} name={icon.name} />,
                  value: icon.value,
                }))}
                selectedValue={<EmployeeLabel color={owner.color} Icon={owner.selectedIcon.icon} name={owner.selectedIcon.name} />}
                onChange={(value) => {
                  owner.icon = value ?? fallbackIcon
                }}
              />
            </div>
            <LabeledInput autoComplete="email" label="Email" type="email" value={owner.email} onChange={(value) => (owner.email = value)} />
            <LabeledInput
              autoComplete="new-password"
              label="Password"
              type="password"
              value={owner.password}
              onChange={(value) => (owner.password = value)}
            />
            {error ? <p className="text-sm text-destructive">{error}</p> : null}
          </CardContent>
          <CardFooter className="justify-end">
            <Button disabled={!owner.isBootstrapOwnerValid || isSaving} type="submit">
              <ArrowRightIcon className="size-4" />
              {isSaving ? (isLoginSetup ? 'Saving...' : 'Creating...') : isLoginSetup ? 'Save login' : 'Continue'}
            </Button>
          </CardFooter>
        </form>
      </Card>
    </AuthShell>
  )
})
