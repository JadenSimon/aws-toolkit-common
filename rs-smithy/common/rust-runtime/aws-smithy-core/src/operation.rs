// Note that 'operations' represent stateful entities that capture IO shapes plus universal context
// 'services' represent _stateless_ implementations of a communication protocol and so they must store or
// infer context from an operation. 
//
// Context is (mostly) an abstract thing and thus needs to support dynamic shapes. So it will need to be
// stored on the heap.
// 
// For context, this is the inverse of how the SDK is currently doing things. The current services 'wrap' 
// other services to create a composite that maps request -> response. But this strategy is too rigid
// since we've tightly coupled _how_ we send the data with _what_ we're sending. Each service shouldn't
// need to know what's inside the payload.

// Implementation note:
// `context` per operation should be separate per transformation
// Transformations will only have access to their respective context
// That is, there is some sort of mediator that allocates slots in the context as it moves around

/* 
struct Operation<T, C, R> {
    payload: T,
    context: C, 
    result: Option<R>,
}
*/

// This is very similar to `tower` except we are more opionated on 
/*
trait Operation {

}
*/


pub struct Operation<T> {
    payload: T,
}

impl<T> Operation<T> {
    pub fn new(payload: T) -> Self {
        Self { payload }
    }

    pub fn get_payload(&self) -> &T {
        &self.payload
    }
}
