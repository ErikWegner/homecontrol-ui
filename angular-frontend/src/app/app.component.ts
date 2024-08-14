import { Component, OnInit, signal } from '@angular/core';
import { RouterModule } from '@angular/router';
import { AuthService, AuthState } from '@homecontrol-ui/shared-ui';

@Component({
  standalone: true,
  imports: [RouterModule],
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrl: './app.component.css',
})
export class AppComponent implements OnInit {
  isAuthenticated = signal(AuthState.Indetermined);
  constructor(private auth: AuthService) {
    this.auth.init();
  }

  ngOnInit(): void {
    this.auth.isAuthenticated().subscribe((authState) => {
      this.isAuthenticated.set(authState);
    });
  }
}
