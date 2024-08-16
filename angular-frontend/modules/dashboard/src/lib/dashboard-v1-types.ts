export interface DashboardV1Command {
  topic: string;
  value: string;
  qos: 0 | 1 | 2;
  retain: boolean;
}

export interface DashboardV1Watch {
  topic: string;
}

export interface DashboardWidgetV1 {
  title: string;
  type: 'button' | 'text';
  icon?: string | null;
  watch?: DashboardV1Watch;
  cmd?: DashboardV1Command;
}

export interface DashboardPanelV1 {
  title: string;
  widgets: DashboardWidgetV1[];
}

export interface DashboardContainerV1 {
  title: string;
  panels: DashboardPanelV1[];
}

export interface DashboardDefinitionV1 {
  containers: DashboardContainerV1[];
}
