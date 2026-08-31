import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom'
import { TooltipProvider } from '@/components/ui/tooltip'
import { Toaster } from '@/components/ui/sonner'
import { ThemeProvider } from '@/components/theme-provider'
import { StudioLayout } from '@/pages/studio/layout'
import { StudioDashboardPage } from '@/pages/studio/dashboard'
import { StudioMoviesPage } from '@/pages/studio/movies-list'
import { StudioMovieDetailPage } from '@/pages/studio/movie-detail'
import { StudioSceneDetailPage } from '@/pages/studio/scene-detail'
import { StudioLicensesPage } from '@/pages/studio/licenses-list'
import { StudioLicenseDetailPage } from '@/pages/studio/license-detail'
import DesignSystemPage from '@/pages/design-system'

export default function App() {
  return (
    <TooltipProvider>
      <ThemeProvider defaultTheme="dark">
        <BrowserRouter>
          <Routes>
            <Route path="/" element={<Navigate to="/studio" replace />} />
            <Route path="/studio" element={<StudioLayout />}>
              <Route index element={<StudioDashboardPage />} />
              <Route path="movies" element={<StudioMoviesPage />} />
              <Route path="movies/:movieId" element={<StudioMovieDetailPage />} />
              <Route path="movies/:movieId/scenes/:sceneId" element={<StudioSceneDetailPage />} />
              <Route path="licenses" element={<StudioLicensesPage />} />
              <Route path="licenses/:licenseId" element={<StudioLicenseDetailPage />} />
            </Route>
            <Route path="/design-system" element={<DesignSystemPage />} />
            <Route path="*" element={<Navigate to="/studio" replace />} />
          </Routes>
        </BrowserRouter>
        <Toaster />
      </ThemeProvider>
    </TooltipProvider>
  )
}
