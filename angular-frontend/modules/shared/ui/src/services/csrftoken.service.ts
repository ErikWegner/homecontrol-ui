import { Injectable } from '@angular/core';

@Injectable({
  providedIn: 'root',
})
export class CsrfTokenService {
  private token: string | null = null;

  public setToken(token: string): void {
    this.token = token;
  }
  public getToken(): string | null {
    return this.token;
  }
}
