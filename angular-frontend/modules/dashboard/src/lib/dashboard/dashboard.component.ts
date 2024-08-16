import { CommonModule } from '@angular/common';
import { Component, OnInit, signal } from '@angular/core';
import { ContainerComponent } from '../components/container/container.component';
import { PanelComponent } from '../components/panel/panel.component';
import { WidgetComponent } from '../components/widget/widget.component';
import { HcBackendService } from '../services/hc-backend.service';
import { DashboardDefinitionV1 } from '../dashboard-v1-types';

@Component({
  selector: 'lib-dashboard',
  standalone: true,
  imports: [CommonModule, ContainerComponent, PanelComponent, WidgetComponent],
  templateUrl: './dashboard.component.html',
  styleUrl: './dashboard.component.css',
})
export class DashboardComponent implements OnInit {
  loading = signal(false);
  errorText = signal<string>('');
  board = signal<DashboardDefinitionV1>({ containers: [] });

  constructor(private readonly hcBackendService: HcBackendService) { }

  ngOnInit(): void {
    this.loading.set(true);
    this.errorText.set('');
    this.board.set({ containers: [] });
    this.hcBackendService.readBoard().subscribe({
      next: (board) => {
        this.loading.set(false);
        this.board.set(board);
      },
      error: (error) => {
        this.loading.set(false);
        this.errorText.set(error.message);
      },
    });
  }
}
