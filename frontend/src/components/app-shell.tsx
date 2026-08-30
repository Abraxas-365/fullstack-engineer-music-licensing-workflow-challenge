import { cn } from '@/lib/utils'
import { Separator } from '@/components/ui/separator'
import { Switch } from '@/components/ui/switch'
import { Label } from '@/components/ui/label'
import { UserAvatar } from '@/components/user-avatar'
import { useTheme } from '@/components/theme-provider'
import type { PlatformRole } from '@/types'

// ─── Sidebar ───
interface NavItem {
  label: string
  href: string
  icon?: React.ReactNode
  count?: number
}

interface SidebarProps {
  items: NavItem[]
  activeHref: string
  onNavigate: (href: string) => void
}

export function AppSidebar({ items, activeHref, onNavigate }: SidebarProps) {
  return (
    <nav className="hidden lg:block w-56 shrink-0 border-r border-border sticky top-14 h-[calc(100vh-3.5rem)] overflow-y-auto py-6 px-3">
      <ul className="space-y-0.5">
        {items.map(item => (
          <li key={item.href}>
            <button
              onClick={() => onNavigate(item.href)}
              className={cn(
                'w-full flex items-center justify-between px-3 py-1.5 rounded-md text-[13px] transition-colors text-left',
                activeHref === item.href
                  ? 'bg-accent text-accent-foreground font-medium'
                  : 'text-muted-foreground hover:text-foreground hover:bg-accent/50',
              )}
            >
              <span className="flex items-center gap-2">
                {item.icon}
                {item.label}
              </span>
              {item.count !== undefined && (
                <span className="text-[11px] text-muted-foreground tabular-nums">
                  {item.count}
                </span>
              )}
            </button>
          </li>
        ))}
      </ul>
    </nav>
  )
}

// ─── Header ───
interface AppHeaderProps {
  userName?: string
  userRole?: PlatformRole
}

export function AppHeader({ userName = 'User', userRole }: AppHeaderProps) {
  const { theme, toggleTheme } = useTheme()

  return (
    <header className="border-b border-border bg-background/80 backdrop-blur-sm sticky top-0 z-50">
      <div className="flex items-center justify-between h-14 px-6">
        <div className="flex items-center gap-3">
          <div className="h-7 w-7 rounded-md bg-primary flex items-center justify-center">
            <span className="text-primary-foreground font-semibold text-xs">ML</span>
          </div>
          <span className="font-semibold text-[15px] tracking-tight">Music Licensing</span>
        </div>
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-2">
            <Label htmlFor="theme" className="text-xs text-muted-foreground">
              {theme === 'dark' ? 'Dark' : 'Light'}
            </Label>
            <Switch id="theme" checked={theme === 'dark'} onCheckedChange={toggleTheme} />
          </div>
          <Separator orientation="vertical" className="h-4" />
          <UserAvatar name={userName} role={userRole} size="sm" />
        </div>
      </div>
    </header>
  )
}

// ─── Shell ───
interface AppShellProps {
  sidebar?: React.ReactNode
  children: React.ReactNode
}

export function AppShell({ sidebar, children }: AppShellProps) {
  return (
    <div className="min-h-screen bg-background text-foreground">
      <div className="flex">
        {sidebar}
        <main className="flex-1 min-w-0 px-8 py-8">
          {children}
        </main>
      </div>
    </div>
  )
}
