import { TestBed } from '@angular/core/testing';

import { HcBackendService } from './hc-backend.service';

describe('HcBackendService', () => {
  let service: HcBackendService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(HcBackendService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
