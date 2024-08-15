import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Web2MqttPayload } from '../web2mqtt-payload';
import { webSocket, WebSocketSubject } from 'rxjs/webSocket';
import { ReplaySubject } from 'rxjs';

@Injectable({
  providedIn: 'root',
})
export class HcBackendService {
  ws: WebSocketSubject<Record<string, unknown>>;

  private topicListeners: Map<string, ReplaySubject<string>> = new Map();

  constructor(private http: HttpClient) {
    this.ws = webSocket({
      url: 'ws://localhost:3020/api/ws',
      deserializer: (e) => {
        try {
          return JSON.parse(e.data);
        } catch (e) {
          /* */
        }
        return { msg: e.data };
      },
    });
    this.ws.subscribe({
      next: (msg) => {
        try {
          if ('type' in msg) {
            const type = msg['type'];
            if (type == 'update') {
              const topic = msg['topic'] as string;
              const data = msg['data'] as string;

              if (topic && data) {
                this.topicListeners.get(topic)?.next(data);
              }
            }
          }
        } catch (e) {
          console.error('Error parsing JSON message:', e);
        }
      }, // Called whenever there is a message from the server.
      error: (err) => console.log(err), // Called if at any point WebSocket API signals some kind of error.
      complete: () => console.log('complete'), // Called when connection is closed (for whatever reason).
    });
  }

  public publish(o: { payload: Web2MqttPayload }) {
    return this.http.post('/api/publish', o.payload, {
      responseType: 'text',
    });
  }

  public subscribe(o: { topic: string }) {
    this.ws.next({
      cmd: 'sub',
      topic: o.topic,
    });

    if (!this.topicListeners.has(o.topic)) {
      this.topicListeners.set(o.topic, new ReplaySubject<string>(1));
    }
    return this.topicListeners.get(o.topic)?.asObservable();
  }
}
