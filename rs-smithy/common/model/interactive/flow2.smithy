$version: "1.0"

namespace aws.toolkits.flow

//use smithy.api#trait
//use smithy.api#idRef

/// This is used as a mechanism for defining type bounds on configuration items.
@trait
@idRef(failWhenMissing: true)
string configType

/// Creates a 'local' generic within the context of the struct
@trait
@idRef(failWhenMissing: false)
string generic


@aws.toolkits.samcli#jsonrpc(version: "2.0")
service FlowService {
    version: "0.0.1",
    resources: [Flow, aws.toolkits.core#SpawnedTool],
    operations: [
        aws.toolkits.core#GetResources, 
        aws.toolkits.core#GetFeatures,
        aws.toolkits.core#RunFeature,
        
        // !!! NOTE !!!
        // I'm adding this to the 'frontend' service because it's the easiest way to prototype
        // this doesn't belong here at all
        aws.toolkits.samcli#CreatePipelineStage,
        aws.toolkits.samcli#RunPipelineBootstrap,
    ],
}

string FlowId

resource Flow {
    identifiers: {
        id: FlowId,
        // TODO: add state token?
        // We should be able to differentiate not just between an 'instance' of a flow but also
        // the various changes that happen to it. 
    },
    create: StartFlow,
    read: GetFlowState,
    update: UpdateFlowState,
    operations: [CompleteFlow, GetFlowSchema],
}


// Some level of mis-direction is required to make this work correctly
// Clients do not directly create resources, they instead request to create them
// The creation flow can then be consumed to finalize creation (think builder pattern)
// Note that there really isn't a difference between 'flows' besides what the final outcome is
// We are always asking the client to make a series of prompts, agnostic to the result
operation StartFlow {
    input: StartFlowInput,
    output: StartFlowOutput,
}

structure StartFlowInput {
    @required
    // To be clear, this should be typed as targeting a ShapeID, 
    target: String,
}

structure StartFlowOutput {
    @required
    id: FlowId,

    @required
    schema: FlowSchema,

    // It is assumed that state is always 'empty' initially, at least from the client's perspective
    // The schema may include a 'default' value for some things, 
}

map FlowState {
    key: SchemaKey,
    /// Value is a serialized form of the correct type
    value: String,
}

@readonly
operation GetFlowState {
    input: GetFlowStateInput,
}

structure GetFlowStateInput {
    @required 
    id: FlowId,
}

structure GetFlowStateOutput {
    @required 
    state: FlowState,
}

string SchemaKey
// This is a superset of `ResourceId`
string TypeId

map FlowSchema {
    key: SchemaKey,
    value: FlowSchemaElement,
}

@uniqueItems
list ValidOptions {
    member: String,
}

// ------------- Templating ------------- //
//
// Templating is a _very_ common way to create dynamic UIs without needing to worry about flow control
// It also describes a common language as opposed to hiding everything on the server
// We could have a lightweight DSL to template 'flows' based off resources
// UIs can then still be extremely dynamic without sacrificing the abstraction
// Plus templating is in pretty much every language, so that's a win for dev productivity. 
//
// Maybe something like this:
//  
//  Option Foo { name: `foo${Bar}`, resourceType: FooType };
//  Option Bar { name: "bar", resourceType: String };
//
// -------------------------------------- //

// XXX: these shapes need to be generated
//string T 
//@generic(T)
structure FlowSchemaElement {
    /// User-friendly name of the 'question'. This is generally a word or short phrase.
    @required
    name: String,

    @required
    // For now, we will not try to provide 'hints' for resource types
    // This generally is an async operation which probably means another client call to resolve it
    /// This can either be a primitive type or a reference to another resource
    resourceType: TypeId,

    /// If true, the client must_ensure this value has been set
    required: Boolean,

    /// Description of the question. This may be user-facing.
    description: String,

    /// This must be the same type as the desired input.
    /// It is currently assumed to be a serialized form of 'TypeId'
    defaultValue: String,

    // Undecided if `validOptions` should be included or not
    // The majority of validation would happen on the server, so its value would be that it acts
    // as an anonymous enum type. Which ain't too bad.
    validOptions: ValidOptions,

    // Needed to represent an option being 'disabled' as opposed to not present at all
    disabled: Boolean,

    // XXX: this is temporary until I figure out a good way to describe dependencies and/or structure
    relativeOrder: Integer,
}

// State updates _must_ occur 1 key/value pair at a time
// This is because the schema _might_ change depending on user input
@idempotent
operation UpdateFlowState {
    input: UpdateFlowInput,
    output: UpdateFlowOutput,
    errors: [InvalidValueError],
}

structure UpdateFlowInput {
    @required 
    id: FlowId,

    @required
    key: SchemaKey,

    @required
    value: String,
}

structure UpdateFlowOutput {
    // TODO: add schema
}

@error("client")
structure InvalidValueError {
    @required
    message: String,
}

// In some sense, completeting a flow is a lifecycle operation since the resource is 'consumed' by
// delivering the result to a handler. The handler populates the output.
// TODO: this should put the flow in a 'completed' state
@idempotent
operation CompleteFlow {
    input: CompleteFlowInput,
    output: CompleteFlowOutput,
}

structure CompleteFlowInput {
    @required 
    id: FlowId,
}

structure CompleteFlowOutput {
    // TODO: the current `data` type is a placeholder
    @required
    data: String, 
}

@readonly
operation GetFlowSchema {
    input: GetFlowSchemaInput,
    output: GetFlowSchemaOutput,
}

structure GetFlowSchemaInput {
    @required 
    id: FlowId,
}

structure GetFlowSchemaOutput {
    @required
    schema: FlowSchema,
}