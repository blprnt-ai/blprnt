import { MoonIcon, SunIcon } from 'lucide-react'
import { useTheme } from 'next-themes'
import { useModeAnimation } from 'react-theme-switch-animation'
import { Button } from '../ui/button'

export const ThemeToggle = () => {
  const { resolvedTheme, setTheme } = useTheme()
  const isDarkMode = resolvedTheme === 'dark'

  const { ref, toggleSwitchTheme } = useModeAnimation({
    isDarkMode,
    onDarkModeChange: (isDark) => {
      setTheme(isDark ? 'dark' : 'light')
    },
  })

  return (
    <Button ref={ref} size="icon" variant="ghost" onClick={toggleSwitchTheme}>
      {!isDarkMode ? <SunIcon className="h-4 w-4" /> : <MoonIcon className="h-4 w-4" />}
    </Button>
  )
}
