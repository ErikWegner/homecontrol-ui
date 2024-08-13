import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterLink } from '@angular/router';
import { AuthService } from '../../services/auth.service';

@Component({
  selector: 'lib-welcome-page',
  standalone: true,
  imports: [CommonModule, RouterLink],
  templateUrl: './welcome-page.component.html',
  styleUrl: './welcome-page.component.css',
})
export class WelcomePageComponent {
  constructor(private auth: AuthService) {}
  public login() {
    this.auth.login();
  }
}
