export interface Web2MqttPayload {
  topic: string;
  value: string;
  qos: 0 | 1 | 2;
  retain: boolean;
}
