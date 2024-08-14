import { Route } from '@angular/router';
import { DashboardComponent } from '@homecontrol-ui/dashboard';
import {
  authenticatedGuard,
  NotFoundPageComponent,
  WelcomePageComponent,
} from '@homecontrol-ui/shared-ui';

export const appRoutes: Route[] = [
  {
    path: '',
    component: WelcomePageComponent,
  },
  {
    path: 'o',
    component: DashboardComponent,
    canActivate: [
      // AuthenticatedGuard
      authenticatedGuard,
    ],
  },
  {
    path: '**',
    component: NotFoundPageComponent,
  },
];
