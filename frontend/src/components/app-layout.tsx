import { BriefcaseBusiness, Check, ChevronsUpDown, LogOut, Moon, Radio, Sun } from 'lucide-react'
import { Link, useNavigate } from 'react-router-dom'
import { useAuth } from '@/lib/auth'
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarInset,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarProvider,
  SidebarRail,
  SidebarSeparator,
  SidebarTrigger,
  useSidebar,
} from '@/components/ui/sidebar'
import { Separator } from '@/components/ui/separator'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { UserAvatar } from '@/components/user-avatar'
import { NotificationBell } from '@/components/notification-bell'
import { useApiMode } from '@/api/use-api-mode'
import { useTheme } from '@/components/theme-provider'
import {
  RIGHTS_PERSONAS,
  setRightsPersona,
  type RightsPersonaId,
} from '@/lib/rights-persona'
import type { PlatformRole } from '@/types'

interface NavItem {
  label: string
  href: string
  icon?: React.ReactNode
  count?: number
}

interface NavGroup {
  label: string
  items: NavItem[]
}

/** Account menu pinned to the sidebar footer: identity + dev/theme controls. */
function UserMenu({
  userName = 'User',
  userRole,
  activePersona,
}: {
  userName?: string
  userRole?: PlatformRole
  activePersona?: RightsPersonaId | 'studio'
}) {
  const { theme, toggleTheme } = useTheme()
  const [apiMode, setApiMode] = useApiMode()
  const { logout } = useAuth()
  const navigate = useNavigate()

  function selectWorkspace(persona: RightsPersonaId | 'studio') {
    if (persona === 'studio') {
      navigate('/studio')
      return
    }
    setRightsPersona(persona)
    navigate('/rights')
  }

  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        render={
          <SidebarMenuButton size="lg" className="data-open:bg-sidebar-accent data-open:text-sidebar-accent-foreground" />
        }
      >
        <UserAvatar name={userName} role={userRole} size="sm" />
        <span className="min-w-0 flex-1">
          <span className="block truncate text-[13px] font-medium">{userName}</span>
          {userRole && <span className="block truncate text-[11px] text-muted-foreground">{userRole}</span>}
        </span>
        <ChevronsUpDown className="ml-auto size-3.5 shrink-0 text-muted-foreground" />
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" side="top" className="w-56">
        <DropdownMenuGroup>
          <DropdownMenuLabel>My account</DropdownMenuLabel>
        </DropdownMenuGroup>
        <DropdownMenuSeparator />
        <DropdownMenuGroup>
          <DropdownMenuLabel>Switch workspace</DropdownMenuLabel>
          <DropdownMenuItem onClick={() => selectWorkspace('studio')} className="justify-between">
            <span className="flex items-center gap-1.5"><BriefcaseBusiness /> Studio team</span>
            {activePersona === 'studio' && <Check className="size-3.5" />}
          </DropdownMenuItem>
          {(Object.values(RIGHTS_PERSONAS) as Array<(typeof RIGHTS_PERSONAS)[RightsPersonaId]>).map(persona => (
            <DropdownMenuItem key={persona.id} onClick={() => selectWorkspace(persona.id)} className="justify-between">
              <span className="min-w-0">
                <span className="block truncate">{persona.kind === 'independent' ? 'Independent artist' : persona.labelRole === 'OWNER' ? 'Label owner' : persona.labelRole === 'REP' ? 'Label rep' : 'Label artist'}</span>
                <span className="block truncate text-[10px] text-muted-foreground">{persona.user.name} · {persona.title}</span>
              </span>
              {activePersona === persona.id && <Check className="size-3.5 shrink-0" />}
            </DropdownMenuItem>
          ))}
        </DropdownMenuGroup>
        <DropdownMenuSeparator />
        <DropdownMenuItem closeOnClick={false} onClick={toggleTheme} className="justify-between">
          <span className="flex items-center gap-1.5">
            {theme === 'dark' ? <Moon /> : <Sun />}
            Theme
          </span>
          <span className="text-[11px] text-muted-foreground capitalize">{theme}</span>
        </DropdownMenuItem>
        <DropdownMenuItem
          closeOnClick={false}
          onClick={() => setApiMode(apiMode === 'real' ? 'mock' : 'real')}
          className="justify-between"
        >
          <span className="flex items-center gap-1.5">
            <Radio />
            API source
          </span>
          <span className="text-[11px] text-muted-foreground">{apiMode === 'real' ? 'Live' : 'Mock'}</span>
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem onClick={async () => { await logout(); navigate('/login') }}>
          <LogOut />
          Sign out
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}

interface AppLayoutProps {
  navGroups: NavGroup[]
  activeHref: string
  userName?: string
  userRole?: PlatformRole
  activePersona?: RightsPersonaId | 'studio'
  workspaceHref?: string
  workspaceName?: string
  workspaceDescription?: string
  headerLabel?: string
  children: React.ReactNode
}

function NavGroups({ navGroups, activeHref }: { navGroups: NavGroup[]; activeHref: string }) {
  const { setOpenMobile } = useSidebar()

  return (
    <>
      {navGroups.map(group => (
        <SidebarGroup key={group.label}>
          <SidebarGroupLabel>{group.label}</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {group.items.map(item => (
                <SidebarMenuItem key={item.href}>
                  <SidebarMenuButton
                    isActive={item.href === activeHref}
                    tooltip={item.label}
                    onClick={() => setOpenMobile(false)}
                    render={<Link to={item.href} />}
                  >
                    {item.icon}
                    <span>{item.label}</span>
                    {item.count !== undefined && (
                      <span className="ml-auto text-[11px] text-muted-foreground tabular-nums">{item.count}</span>
                    )}
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      ))}
    </>
  )
}

export function AppLayout({
  navGroups,
  activeHref,
  userName,
  userRole,
  activePersona,
  workspaceHref = '/studio',
  workspaceName = 'Music Licensing',
  workspaceDescription = 'Studio workspace',
  headerLabel,
  children,
}: AppLayoutProps) {
  const { theme, toggleTheme } = useTheme()

  return (
    <SidebarProvider>
      <Sidebar variant="inset">
        <SidebarHeader>
          <SidebarMenu>
            <SidebarMenuItem>
              <SidebarMenuButton size="lg" render={<Link to={workspaceHref} />}>
                <div className="flex h-7 w-7 items-center justify-center rounded-md bg-primary text-primary-foreground text-xs font-semibold">
                  ML
                </div>
                <div className="min-w-0 flex flex-col">
                  <span className="truncate text-sm font-semibold tracking-tight">{workspaceName}</span>
                  <span className="truncate text-[10px] text-muted-foreground">{workspaceDescription}</span>
                </div>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarHeader>

        <SidebarSeparator />

        <SidebarContent>
          <NavGroups navGroups={navGroups} activeHref={activeHref} />
        </SidebarContent>

        <SidebarFooter>
          <SidebarMenu>
            <SidebarMenuItem>
              <UserMenu userName={userName} userRole={userRole} activePersona={activePersona} />
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarFooter>

        <SidebarRail />
      </Sidebar>

      <SidebarInset>
        <header className="flex h-14 items-center justify-between gap-2 border-b border-border px-4 sm:px-6">
          <div className="flex items-center gap-2">
            <SidebarTrigger className="-ml-1" />
            <Separator orientation="vertical" className="mr-1 !h-4" />
            {headerLabel && <span className="text-[13px] text-muted-foreground">{headerLabel}</span>}
          </div>
          <div className="flex items-center gap-1 sm:gap-2">
            <NotificationBell />
            <Button variant="ghost" size="icon" aria-label="Toggle theme" onClick={toggleTheme}>
              {theme === 'dark' ? <Moon className="size-4" /> : <Sun className="size-4" />}
            </Button>
          </div>
        </header>
        <main className="flex-1 min-w-0 px-4 py-6 sm:px-6 lg:px-8 lg:py-8">
          {children}
        </main>
      </SidebarInset>
    </SidebarProvider>
  )
}
