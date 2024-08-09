import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Web2MqttPayload } from '../web2mqtt-payload';
import { webSocket } from 'rxjs/webSocket';

@Injectable({
  providedIn: 'root',
})
export class HcBackendService {
  constructor(private http: HttpClient) {
    const ws = webSocket({
      url: 'ws://localhost:3000/api/ws',
      deserializer: (e) => e.data,
    });
    ws.subscribe({
      next: (msg) => console.log('message received: ' + msg), // Called whenever there is a message from the server.
      error: (err) => console.log(err), // Called if at any point WebSocket API signals some kind of error.
      complete: () => console.log('complete'), // Called when connection is closed (for whatever reason).
    });
  }

  public publish(o: { payload: Web2MqttPayload }) {
    return this.http.post('/api/publish', o.payload, {
      responseType: 'text',
    });
  }
}
