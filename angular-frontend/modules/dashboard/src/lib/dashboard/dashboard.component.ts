import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';
import { ContainerComponent } from '../components/container/container.component';
import { PanelComponent } from '../components/panel/panel.component';
import { WidgetComponent } from '../components/widget/widget.component';

@Component({
  selector: 'lib-dashboard',
  standalone: true,
  imports: [CommonModule, ContainerComponent, PanelComponent, WidgetComponent],
  templateUrl: './dashboard.component.html',
  styleUrl: './dashboard.component.css',
})
export class DashboardComponent { }
