use std::sync::Arc;
use async_trait::async_trait;
use toolkits::model::ResourceSummary;

// TODO: MOVE THESE TO SOMEWHERE COMMON //
// -------------- Traits -------------- //

// Where does this belong? 
// This is more of a 'feature' for the service, not the child resource
// Smithy models resources with the 'list' operation baked-in, but it feels out of place.
// TODO: add `Resource` bounds?
#[async_trait]
pub trait List<T> {
    type Error;

    async fn list(&self) -> Result<Vec<Arc<T>>, Self::Error>;
}

#[async_trait]
pub trait Create<T> {
    type Input;
    type Error;

    async fn create(&mut self, input: Self::Input) -> Result<Arc<T>, Self::Error>;
}

// !!!!!!!!TODO!!!!!!!! NOT IMPLEMENTED
// Not sure how to structure this feature
// It describes how to poll a Stateful resource
/// Pollers watch a collection of resources, emitting events when things change. (TODO: figure this out)
#[async_trait]
pub trait Poller<T: Stateful> {
    type Input;
    type Error;

    async fn poll(&mut self, input: Self::Input) -> Result<Arc<T>, Self::Error>;
}

/// Registries keep track of resources. It's assumed that the target resource has already been
/// derived through some other mechanism, so an API call is unnecessary.  
/// 
/// Later on we may want the idea of a 'lazy' registry that doesn't need to list prior to checking.
/// This, of course, would be async.
#[async_trait]
pub trait Registry<T: Resource> {
    // TODO: we can get rid of all the 'Arc' with smart pointers
    async fn get_resource(&self, iri: &str) -> Option<Arc<T>>;
}

/// Describes a generic 'resource' object
pub trait Resource {
    fn summary(&self) -> ResourceSummary;
}

pub trait Stateful {
    /// This is intended to be a user-friendly string about the current state.
    fn get_state(&self) -> String;

    /// Resources are only transient if they are not in a pre-determined steady-state.
    /// For now we'll ignore any scenarios where resources are transient based off other resources.
    fn is_transient(&self) -> bool;
}
