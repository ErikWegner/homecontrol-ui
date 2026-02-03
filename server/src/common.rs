use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Dashboard {
    pub title: String,
    #[serde(rename = "version")]
    pub version_number: u32,
    pub pages: Vec<Page>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    pub name: String,
    pub groups: Vec<Group>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Group {
    pub title: String,
    pub rows: Vec<Row>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Row {
    pub controls: Vec<Control>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Control {
    Button {
        label: String,
        action: String,
        mqtt_instance_identifier: String,
        mqtt_topic: String,
        mqtt_json_path: String,
    },
    GaugeDisplay {
        value: f64,
        unit: String,
        mqtt_instance_identifier: String,
        mqtt_topic: String,
        mqtt_json_path: String,
    },
}
