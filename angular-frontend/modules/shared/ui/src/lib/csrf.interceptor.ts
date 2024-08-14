import { HttpInterceptorFn } from '@angular/common/http';
import { inject } from '@angular/core';
import { CsrfTokenService } from '../services/csrftoken.service';

export const csrfInterceptor: HttpInterceptorFn = (req, next) => {
  const csrfToken = inject(CsrfTokenService).getToken() ?? '';

  const newReq = req.clone({
    headers: req.headers.append('x-csrf-token', csrfToken),
  });
  return next(newReq);
};
