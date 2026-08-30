import { Switch } from '@/components/ui/switch'
import { Label } from '@/components/ui/label'
import { cn } from '@/lib/utils'
import { useApiMode } from '@/api/use-api-mode'

interface ApiModeToggleProps {
  className?: string
}

/** Dev-facing switch between the mocked API and the real backend. */
export function ApiModeToggle({ className }: ApiModeToggleProps) {
  const [mode, setMode] = useApiMode()

  return (
    <div className={cn('flex items-center gap-2', className)}>
      <Label htmlFor="api-mode-toggle" className="text-[12px] text-muted-foreground">
        {mode === 'mock' ? 'Mock API' : 'Live API'}
      </Label>
      <Switch
        id="api-mode-toggle"
        size="sm"
        checked={mode === 'real'}
        onCheckedChange={checked => setMode(checked ? 'real' : 'mock')}
      />
    </div>
  )
}
