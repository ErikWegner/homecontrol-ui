import { DOCUMENT } from '@angular/common';
import { Inject, Injectable } from '@angular/core';

@Injectable({
  providedIn: 'root',
})
export class AuthService {
  window: Window;

  constructor(@Inject(DOCUMENT) document: Document) {
    this.window = document.defaultView as unknown as Window;
  }

  public login() {
    const oidCallbackUrl = window.location.origin + '/auth/callback';
    const appCallbackUrl = window.location.origin + '/';
    const state =
      Math.random().toString(36).substring(2, 15) +
      '-appstate-' +
      Math.random().toString(36).substring(2, 15);

    sessionStorage.setItem('state', state);

    const params = Object.entries({
      scope: 'openid',
      redirect_uri: oidCallbackUrl,
      app_uri: appCallbackUrl,
      state,
    })
      .map(([key, value]) => key + '=' + encodeURIComponent(value))
      .join('&');

    this.window.location = '/auth/login?' + params;
  }
}
