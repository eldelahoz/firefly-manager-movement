import { createContext, useContext, useEffect, useState } from 'react';
import { Toaster } from 'sonner';

type Theme = 'light' | 'dark';
const ThemeContext = createContext<{
  theme: Theme;
  toggle: () => void;
  setTheme: (t: Theme) => void;
}>({
  theme: 'light',
  toggle: () => {},
  setTheme: () => {},
});

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const [theme, setTheme] = useState<Theme>(() => {
    const saved = localStorage.getItem('ffm.theme');
    if (saved === 'light' || saved === 'dark') return saved;
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  });
  useEffect(() => {
    document.documentElement.classList.toggle('dark', theme === 'dark');
    localStorage.setItem('ffm.theme', theme);
  }, [theme]);
  return (
    <ThemeContext.Provider
      value={{
        theme,
        toggle: () => setTheme((t) => (t === 'dark' ? 'light' : 'dark')),
        setTheme: (t) => setTheme(t),
      }}
    >
      {children}
      {/* Offsets clear the 64px mobile nav (shown below md, wider than sonner's 600px breakpoint). */}
      <Toaster theme={theme} richColors offset={{ bottom: 80 }} mobileOffset={{ bottom: 72 }} />
    </ThemeContext.Provider>
  );
}

export const useTheme = () => useContext(ThemeContext);
