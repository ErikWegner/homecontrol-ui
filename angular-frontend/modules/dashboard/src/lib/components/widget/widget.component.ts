import { CommonModule } from '@angular/common';
import {
  AfterViewChecked,
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
export class WidgetComponent implements AfterViewChecked {
  type = input<WidgetType>('text');
  title = input('');
  icon = input('icons/fullcircle.svg');
  value = signal('');
  cmd = input<Web2MqttPayload | null>(null);
  watch = input<Web2MqttWatch | null>(null);
  checkFontSize = false;

  @ViewChild('textscale') textscale: ElementRef | null = null;

  ngAfterViewChecked() {
    if (this.checkFontSize) {
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
    }
  }

  constructor(private hc: HcBackendService) {
    let lastValue = '';
    effect(() => {
      const currentValue = this.value();
      if (currentValue !== lastValue) {
        lastValue = currentValue;
        this.checkFontSize = true;
      }
    });
    effect(() => {
      const watch = this.watch();
      if (!watch) {
        return;
      }
      this.hc.subscribe(watch)?.subscribe((v) => {
        if (watch.suffix && typeof watch.suffix === 'string') {
          v = v + watch.suffix;
        }
        this.value.set(v);
      });
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
