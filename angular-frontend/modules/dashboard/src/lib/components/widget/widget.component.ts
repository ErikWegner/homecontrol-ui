import { Component, input } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'lib-widget',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './widget.component.html',
  styleUrl: './widget.component.css',
})
export class WidgetComponent {
  title = input('');
}
