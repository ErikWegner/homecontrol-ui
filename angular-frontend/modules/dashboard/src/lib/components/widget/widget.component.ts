import { CommonModule } from '@angular/common';
import {
  Component,
  effect,
  ElementRef,
  input,
  signal,
  ViewChild,
} from '@angular/core';
import { HcBackendService } from '../../services/hc-backend.service';
import { Web2MqttPayload } from '../../web2mqtt-payload';
import { Web2MqttWatch } from '../../web2mqtt-watch';
import { WidgetType } from '../../widget-type';

@Component({
  selector: 'lib-widget',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './widget.component.html',
  styleUrl: './widget.component.css',
})
export class WidgetComponent {
  type = input<WidgetType>('text');
  title = input('');
  icon = input('icons/fullcircle.svg');
  value = signal('');
  cmd = input<Web2MqttPayload | null>(null);
  watch = input<Web2MqttWatch | null>(null);

  @ViewChild('textscale') textscale: ElementRef | null = null;

  constructor(private hc: HcBackendService) {
    effect(() => {
      const desiredWidth = 120;
      const scaleFontContainer = this.textscale?.nativeElement;
      if (scaleFontContainer) {
        scaleFontContainer.style.fontSize = '96px';
        let fontSize = parseInt(
          window
            .getComputedStyle(scaleFontContainer, null)
            .getPropertyValue('font-size')
        );
        while (scaleFontContainer.scrollWidth > desiredWidth && fontSize > 6) {
          fontSize--;
          scaleFontContainer.style.fontSize = fontSize + 'px';
        }
      }
    });
    effect(() => {
      const watch = this.watch();
      console.log('watch:', watch);
      if (!watch) {
        return;
      }
      this.hc.subscribe(watch)?.subscribe((v) => this.value.set(v));
    });
  }

  onClick() {
    const payload = this.cmd();
    if (!payload) {
      return;
    }
    this.hc.publish({ payload }).subscribe();
  }
}
