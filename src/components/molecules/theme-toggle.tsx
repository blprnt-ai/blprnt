import { MoonIcon, SunIcon } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useModeAnimation } from 'react-theme-switch-animation'
import { Button } from '../ui/button'

export const ThemeToggle = () => {
  const [isLoaded, setIsLoaded] = useState(false)
  const [isDarkMode, setIsDarkMode] = useState(true)

  const { ref, toggleSwitchTheme } = useModeAnimation({
    isDarkMode,
    onDarkModeChange: (isDark) => {
      setIsDarkMode(isDark)
    },
  })

  useEffect(() => {
    const previousIsDarkMode = localStorage.getItem('isDarkMode')

    if (previousIsDarkMode) {
      setIsDarkMode(previousIsDarkMode === 'true')
    } else {
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)')
      setIsDarkMode(prefersDark.matches)
    }
    setIsLoaded(true)
  }, [])

  // biome-ignore lint/correctness/useExhaustiveDependencies: Do not react to isLoaded changes
  useEffect(() => {
    if (!isLoaded) return
    localStorage.setItem('isDarkMode', isDarkMode.toString())
  }, [isDarkMode])

  return (
    <Button ref={ref} size="icon" variant="ghost" onClick={toggleSwitchTheme}>
      {isDarkMode ? <SunIcon className="h-4 w-4" /> : <MoonIcon className="h-4 w-4" />}
    </Button>
  )
}
