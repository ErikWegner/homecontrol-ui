import { Route } from '@angular/router';
import {
  NotFoundPageComponent,
  WelcomePageComponent,
} from '@homecontrol-ui/shared-ui';
import { DashboardComponent } from '@homecontrol-ui/dashboard';

export const appRoutes: Route[] = [
  {
    path: '',
    component: WelcomePageComponent,
  },
  {
    path: 'o',
    component: DashboardComponent,
  },
  {
    path: '**',
    component: NotFoundPageComponent,
  },
];
