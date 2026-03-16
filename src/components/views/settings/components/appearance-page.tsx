import { useEffect, useState } from 'react'
import { Section } from '@/components/organisms/page/section'
import { SectionField } from '@/components/organisms/page/section-field'

export const AppearancePage = () => {
  return (
    <Section>
      <SectionField title="Theme">
        <ThemeSwitch />
      </SectionField>
    </Section>
  )
}
const ThemeSwitch = () => {
  const [theme, setTheme] = useState<'light' | 'dark'>((localStorage.getItem('theme') as 'light' | 'dark') || 'dark')

  useEffect(() => {
    localStorage.setItem('theme', theme)
  }, [theme])

  const handleThemeChange = () => {
    setTheme(theme === 'light' ? 'dark' : 'light')
    document.documentElement.classList.toggle('dark', theme !== 'dark')
  }

  return (
    <div className="theme-switch py-1.25 px-3.5">
      <label className="theme-switch-label">
        <input checked={theme !== 'dark'} className="theme-checkbox" type="checkbox" onChange={handleThemeChange} />
        <span className="theme-slider" />
      </label>
    </div>
  )
}
