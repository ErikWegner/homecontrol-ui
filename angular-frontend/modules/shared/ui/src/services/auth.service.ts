import { DOCUMENT } from '@angular/common';
import { HttpClient } from '@angular/common/http';
import { Inject, Injectable } from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { BehaviorSubject, tap, timer } from 'rxjs';
import { CsrfTokenService } from './csrftoken.service';

export enum AuthState {
  Indetermined,
  Unauthorized,
  Authenticated,
}

@Injectable({
  providedIn: 'root',
})
export class AuthService {
  private isRunning = false;
  private trySSO = true;
  private lastState = AuthState.Indetermined;
  private expireRetry = 0;
  private readonly authBasePath = '/auth';
  private readonly window: Window;
  private readonly isAuthenticated$ = new BehaviorSubject<AuthState>(
    AuthState.Indetermined
  );

  constructor(
    @Inject(DOCUMENT) private readonly document: Document,
    private http: HttpClient,
    private route: ActivatedRoute,
    private router: Router,
    private csrfTokenService: CsrfTokenService
  ) {
    this.window = document.defaultView as unknown as Window;
  }

  public init() {
    if (this.isRunning) {
      return;
    }
    this.isRunning = true;

    timer(15, 60_000)
      .pipe(tap(() => console.debug('Refreshing auth status...')))
      .subscribe(() => this.checkAuthStatus());
  }

  private checkAuthStatus(): void {
    this.http
      .get<{
        authenticated: boolean;
        expires_in: number | null;
        refresh_expires_in: number | null;
      }>(this.authBasePath + '/status')
      .subscribe((status) => {
        if (status.authenticated) {
          if ((status.expires_in ?? 0) > 12) {
            this.expireRetry = 0;
          }
          if (this.lastState !== AuthState.Authenticated) {
            this.lastState = AuthState.Authenticated;
            this.getCsrfTokenAndUpdateStatus();
          }
          if (
            (status.expires_in ?? 0) < 12 &&
            (status.refresh_expires_in ?? 0) > 0
          ) {
            this.expireRetry++;
            this.http
              .post<string>(this.authBasePath + '/refresh', {})
              .subscribe({
                next: () => {
                  if (this.expireRetry < 4) {
                    this.checkAuthStatus();
                  }
                },
              });
          }
        } else {
          if (this.trySSO) {
            this.trySSO = false;
            this.checkSSO();
          } else {
            if (this.lastState !== AuthState.Unauthorized) {
              this.lastState = AuthState.Unauthorized;
              this.isAuthenticated$.next(AuthState.Unauthorized);
            }
          }
        }
      });
  }

  private checkSSO(): void {
    const f = this.document.createElement('iframe');
    f.src = this.loginUrl({ prompt: 'none' });
    f.setAttribute(
      'sandbox',
      'allow-storage-access-by-user-activation allow-scripts allow-same-origin'
    );
    f.setAttribute('title', 'check-sso');
    f.style.display = 'none';
    document.body.appendChild(f);

    const messageCallback = (event: MessageEvent<string>) => {
      if (
        event.origin !== window.location.origin ||
        f.contentWindow !== event.source
      ) {
        return;
      }

      if ((event.data || 'error').includes('error')) {
        this.isAuthenticated$.next(AuthState.Unauthorized);
      } else {
        this.getCsrfTokenAndUpdateStatus();
      }

      document.body.removeChild(f);
      window.removeEventListener('message', messageCallback);
    };

    window.addEventListener('message', messageCallback);
  }

  private getCsrfTokenAndUpdateStatus() {
    this.http
      .post<{ token: string }>(this.authBasePath + '/csrftoken', {})
      .subscribe((token) => {
        this.csrfTokenService.setToken(token.token);
        this.isAuthenticated$.next(AuthState.Authenticated);
        const returnUrl = this.route.snapshot.queryParamMap.get('returnUrl');
        if (returnUrl) {
          this.router.navigateByUrl(returnUrl);
        }
      });
  }

  public isAuthenticated() {
    return this.isAuthenticated$.asObservable();
  }

  private loginUrl(
    o: { prompt?: 'login' | 'none'; returnUrl?: string } = {
      prompt: 'login',
    }
  ) {
    const oidCallbackUrl = window.location.origin + '/auth/callback';
    const appCallbackUrl =
      window.location.origin +
      (o.prompt == 'none'
        ? '/sso.html'
        : '/' +
          (o.returnUrl ? '?returnUrl=' + encodeURIComponent(o.returnUrl) : ''));
    const state =
      Math.random().toString(36).substring(2, 15) +
      '-appstate-' +
      Math.random().toString(36).substring(2, 15);

    sessionStorage.setItem('state', state);

    const params = Object.entries(<Record<string, string>>{
      scope: 'openid',
      redirect_uri: oidCallbackUrl,
      app_uri: appCallbackUrl,
      state,
    });
    if (o.prompt === 'none') {
      params.push(['prompt', 'none']);
    }
    const q = params
      .map(([key, value]) => key + '=' + encodeURIComponent(value))
      .join('&');

    return '/auth/login?' + q;
  }

  public login(returnUrl?: string) {
    this.window.location = this.loginUrl({ returnUrl });
  }
}
