import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { cn } from '@/lib/utils'
import type { Movie } from '@/types'

interface MovieCardProps {
  movie: Movie
  sceneCount?: number
  trackCount?: number
  licensedCount?: number
  onClick?: () => void
  className?: string
}

export function MovieCard({ movie, sceneCount, trackCount, licensedCount, onClick, className }: MovieCardProps) {
  const year = movie.release_year ?? new Date(movie.created_at).getFullYear()

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
        <div className="flex items-start justify-between">
          <div className="min-w-0">
            <CardTitle className="text-sm font-semibold truncate">{movie.title}</CardTitle>
            {movie.director && (
              <CardDescription className="text-[13px]">
                Directed by {movie.director}
              </CardDescription>
            )}
          </div>
          <Badge variant="outline" className="text-[11px] shrink-0 ml-2">{year}</Badge>
        </div>
      </CardHeader>

      {(sceneCount !== undefined || trackCount !== undefined) && (
        <CardContent className="pb-3">
          <div className="flex gap-2 flex-wrap">
            {sceneCount !== undefined && (
              <Badge variant="secondary" className="text-[11px]">
                {sceneCount} scene{sceneCount !== 1 ? 's' : ''}
              </Badge>
            )}
            {trackCount !== undefined && (
              <Badge variant="secondary" className="text-[11px]">
                {trackCount} track{trackCount !== 1 ? 's' : ''}
              </Badge>
            )}
            {licensedCount !== undefined && licensedCount > 0 && (
              <Badge variant="secondary" className="text-[11px]">
                {licensedCount} licensed
              </Badge>
            )}
          </div>
        </CardContent>
      )}

      <CardFooter className="pt-0">
        <span className="text-[11px] text-muted-foreground">
          {new Date(movie.created_at).toLocaleDateString('en-US', {
            month: 'short', day: 'numeric', year: 'numeric',
          })}
        </span>
      </CardFooter>
    </Card>
  )
}
