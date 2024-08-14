import { CanActivateFn } from '@angular/router';
import { AuthService, AuthState } from '../../services/auth.service';
import { inject } from '@angular/core';
import { filter, map } from 'rxjs';

export const authenticatedGuard: CanActivateFn = (route, state) => {
  const authService = inject(AuthService);
  return authService.isAuthenticated().pipe(
    filter((a) => a !== AuthState.Indetermined),
    map((authenticated) => {
      if (authenticated === AuthState.Authenticated) {
        return true;
      }

      authService.login(state.url);

      return false;
    })
  );
};
