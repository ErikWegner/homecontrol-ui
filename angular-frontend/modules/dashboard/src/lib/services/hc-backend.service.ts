import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Web2MqttPayload } from '../web2mqtt-payload';

@Injectable({
  providedIn: 'root',
})
export class HcBackendService {
  constructor(private http: HttpClient) { }

  public publish(o: { payload: Web2MqttPayload }) {
    return this.http.post<string>('/api/publish', o.payload);
  }
}
