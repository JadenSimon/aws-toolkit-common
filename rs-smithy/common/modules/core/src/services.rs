// This is for abstract services in general, not AWS services

// Useful for testing and ergonomics
pub struct ServiceRegistry;

// Requirements (for a resource registry):
// 1. registry should be thread-safe 
// 2. registry should be cheap to Clone
// 3. accesses and mutations should hold any locks for as little as possible