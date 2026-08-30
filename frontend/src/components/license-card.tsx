import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card'
import { cn } from '@/lib/utils'
import { StatusBadge } from '@/components/status-badge'
import type { LicenseRequest, LicenseOffer, LicenseStatus } from '@/types'

function formatCurrency(amount: number | null, currency: string | null): string {
  if (amount == null) return '--'
  const cur = currency ?? 'USD'
  return new Intl.NumberFormat('en-US', { style: 'currency', currency: cur, maximumFractionDigits: 0 }).format(amount)
}

interface LicenseCardProps {
  license: LicenseRequest
  latestOffer?: LicenseOffer | null
  offerCount?: number
  songTitle?: string
  sceneTitle?: string
  resolvedByName?: string
  onClick?: () => void
  className?: string
}

export function LicenseCard({
  license,
  latestOffer,
  offerCount,
  songTitle,
  sceneTitle,
  resolvedByName,
  onClick,
  className,
}: LicenseCardProps) {
  return (
    <Card
      className={cn(
        'group hover:border-muted-foreground/20 transition-colors',
        onClick && 'cursor-pointer',
        className,
      )}
      onClick={onClick}
    >
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm font-semibold">
            License #{license.id.slice(0, 8)}
          </CardTitle>
          <StatusBadge status={license.status as LicenseStatus} />
        </div>
        {(songTitle || sceneTitle) && (
          <CardDescription className="text-[13px]">
            {songTitle}{sceneTitle ? ` → ${sceneTitle}` : ''}
          </CardDescription>
        )}
      </CardHeader>

      {latestOffer && (
        <CardContent className="pb-3 space-y-1.5">
          <div className="flex justify-between text-[13px]">
            <span className="text-muted-foreground">Fee</span>
            <span className="font-medium">
              {formatCurrency(latestOffer.license_fee, latestOffer.currency)}
            </span>
          </div>
          {latestOffer.territory && (
            <div className="flex justify-between text-[13px]">
              <span className="text-muted-foreground">Territory</span>
              <span>{latestOffer.territory}</span>
            </div>
          )}
          <div className="flex justify-between text-[13px]">
            <span className="text-muted-foreground">Exclusive</span>
            <span>{latestOffer.exclusive ? 'Yes' : 'No'}</span>
          </div>
        </CardContent>
      )}

      <CardFooter className="pt-0">
        <span className="text-[11px] text-muted-foreground">
          {offerCount != null ? `${offerCount} offer${offerCount !== 1 ? 's' : ''}` : ''}
          {resolvedByName ? ` · ${license.status === 'APPROVED' ? 'Accepted' : 'Resolved'} by ${resolvedByName}` : ''}
        </span>
      </CardFooter>
    </Card>
  )
}
