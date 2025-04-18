use super::service_state::ServiceState;
use super::service_property::ServiceProperty;

#[derive(Clone)]
pub struct Service {
   name: String,
   description: String,
   state: ServiceState,
   properties: Option<ServiceProperty>
} 

impl Service {
    pub fn new(name: String, description: String, state: ServiceState) -> Self {
        Service { name, description, state, properties: None }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn state(&self) -> &ServiceState {
        &self.state
    }

    pub fn properties(&self) -> Option<&ServiceProperty> {
        self.properties.as_ref()
    }

    pub fn update_properties(&mut self, properties: ServiceProperty){
        self.properties = Some(properties);
    }
}
