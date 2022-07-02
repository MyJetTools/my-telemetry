use crate::TelemetryEvent;

pub struct TelemtryCollector {
    events_to_publish: Option<Vec<TelemetryEvent>>,
}

impl TelemtryCollector {
    pub fn new() -> Self {
        Self {
            events_to_publish: None,
        }
    }

    pub fn write(&mut self, event: TelemetryEvent) {
        if self.events_to_publish.is_none() {
            self.events_to_publish = Some(Vec::new());
        }

        self.events_to_publish.as_mut().unwrap().push(event);
    }

    pub fn get_events(&mut self) -> Option<Vec<TelemetryEvent>> {
        if self.events_to_publish.is_none() {
            return None;
        }

        self.events_to_publish.take()
    }
}
