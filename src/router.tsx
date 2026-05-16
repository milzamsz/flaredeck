/* eslint-disable react-refresh/only-export-components */
import { lazy, Suspense } from 'react'
import { createBrowserRouter } from 'react-router-dom'
import { AppLayout } from '@/components/app-layout'
import Dashboard from '@/pages/Dashboard'

const ConfigPage = lazy(() => import('@/pages/Config'))
const SettingsPage = lazy(() => import('@/pages/Settings'))

function RouteFallback() {
  return (
    <main className="flex h-full items-center justify-center p-6 text-sm text-muted-foreground">
      Loading…
    </main>
  )
}

export const router = createBrowserRouter([
  {
    element: <AppLayout />,
    children: [
      { path: '/', element: <Dashboard /> },
      {
        path: '/config',
        element: (
          <Suspense fallback={<RouteFallback />}>
            <ConfigPage />
          </Suspense>
        ),
      },
      {
        path: '/settings',
        element: (
          <Suspense fallback={<RouteFallback />}>
            <SettingsPage />
          </Suspense>
        ),
      },
    ],
  },
])
