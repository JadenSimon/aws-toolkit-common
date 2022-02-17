$version: "1.0"

namespace aws.toolkits.core

use smithy.api#trait

// This is intended to be extremely abstract in an attempt to answer the following questions:
// 1. What is a 'resource'?
// 2. What can I do with a resource?
// 3. Where can I find a resource?
//
// We are not trying to answer the 'how' here, but instead describing the 'what'
// This is essentially an entry-point for an ontology

// !!!!! IMPORTANT !!!!!! //
// We will need to be able to statically generate content (usually strings) for certain features 
// Some of the IDEs display things based off a static manifest delivered with the toolkit.
// That manifest will need to be partially generated from feature/resource definitions.
// For features/resources that are explicitly defined via IDL this isn't a big deal, but for
// things that are more 'dynamic' this can be a problem.
//
// A good solution for things implemented directly on the backend might be to use Rust annotations,
// extracting definitions out at build time. 

string IRI
string ResourceType

// While Smithy does have a way to describe resources, it doesn't have a good way to describe abstract resources
// In this case we are using Smithy to describe a 'base' schema from which to drive data through

/// An abstraction over a 'resource' that summarizes relevant UI/UX information
/// The resource may not necessarily be from an AWS service
structure ResourceSummary {
    /// A 'human-friendly' identifier for the resource. This field will never change assuming the same IRI; in other words,
    /// the resource name is apart of the IRI.
    @required
    name: String,

    /// An identifier that describes what the resource represents, e.g. 'File', 'LogGroup', 'SAMTemplate', etc.
    /// Types primarily serve as a way to 're-use' schemas.
    @required
    resourceType: ResourceType,

    // Perhaps something like 'aws-toolkits' will be a better scheme. I don't think AWS has a standard IRI format? 
    // Maybe I should propose one?
    /// An 'IRI' (Internationalized Resource Identifier) is essentially a URI that supports UTF-8
    /// Every resource must have a uniform way to locate it. This is intentionally vague to support forward-compatability.
    /// Clients may use this field to fully describe the resource. The server may also support resolution for certain resources.
    /// 
    /// For data structures provided by AWS services, the proposed format is 'aws:[arn]'. Resources without explicit arns will
    /// require a different format as we do not want to conflate synthetic ARNs with standard ones.
    /// This is highly unstable and will likely change. 
    /// 
    /// Aliased IRIs (different identifiers that point to the same resource) are allowed as long as they have different schemes.
    /// For example, an S3 file may have an IRI as `s3://bucket/file/` and another one as `aws-toolkits:arn:aws:s3:::bucket/file`
    /// Note that the former URI is ambigous about which parititon the resource is in, and thus the latter is preferred.
    @required
    iri: IRI,

    /// Additional information about the resource which may be displayed. 
    description: String,

    /// Similar to `description` but more detailed. This is intended to be verbose and not constantly visible.
    detail: String,

    // facets: Map<String, Structure>
}

// I'm essentially describing a 'meta' interface over smithy models that is feature-orientated
// The services become the 'how', but clients shouldn't need to know that.

string FeatureType
string FeatureId

// Note: features will need to be up-front about possible dependencies
// Features could potentially depend on instances of resources or perhaps even other features
// Feaures will need to be described by the client as well. So a 'server' feature may require
// the presence of a 'client' feature (or even resource).
// The combination of the two RDF graphs (client + server) is how the complete API/ABI is defined.

/// A 'feature' describes what you might be able to do with a certain resource type.
///
/// Keep in mind that features might be conditional. The presence of a feature means that
/// it may be used, while the absence means that it may never be used.
structure Feature {
    @required
    featureType: FeatureType,

    /// This is globally unique
    @required
    id: FeatureId,

    /// A feature's name can be used for presentation; e.g. a 'Delete' feature can be shown as 'Delete Resource...'
    @required
    name: String,

    /// Additional information about the feature
    description: String,
}

list Features {
    member: Feature
}

@readonly
operation GetFeatures {
    input: GetFeaturesInput,
    output: GetFeaturesOutput,
}

structure GetFeaturesInput {
    @required
    resourceType: ResourceType,
}

structure GetFeaturesOutput {
    @required
    features: Features,
}

list Resources {
    member: ResourceSummary,
}

@readonly
operation GetResources {
   input: GetResourcesInput,
   output: GetResourcesOutput,
}

structure GetResourcesInput {
    scope: IRI,
}

structure GetResourcesOutput {
    @required
    resources: Resources,
}

// This is a strange one
// The dynamic feature capability means we're doing a form of type erasure at the signature level
// Basically, running a feature on a resource might require arbitrary input. Likewise, the response
// may be arbitrary. The best strategy is to group at the type level. A basic 'Create' feature will
// look the same across many resources since it returns a 'schema' for how to create it.
// This let's us keep feature interfaces uniform since we encapsulate requirements. 
operation RunFeature {
    input: RunFeatureInput,
    output: RunFeatureOutput,
}

structure RunFeatureInput {
    // By requiring an IRI, we've just made it explicit that all features operate directly on instances, not types
    @required
    target: IRI,

    @required
    feature: FeatureId,
}

// Is sticking everything in a single union the right approach?
// I suppose it's fine for now. I mean this is really just an RPC version of dynamic dispatch
// `RunFeature` is basically the equivalent of a vtable
union RunFeatureData {
    list: Resources,
    create: aws.toolkits.flow#StartFlowOutput,

    // I'm thinking we'll need to have a logical separation between facets and features
    // Basically, facets describe additional (or perhaps alternative) views of the resource without
    // necessarily manipulating or acting on the resource in any way
    //
    // To put it another way: facets are implicit, features are explicit.
    // These lines can blur when one starts to use aggregate data for facets.
    // Though, as long as the facet is pure data then I think it fits.
    facet: StatefulResourceFacet,
}

// Facet vs. feature
// Facets describe; features act
// Facets are passive; features are active
//
// One might think of a facet as the fields of a something, while a feature is the methods
// The combination of the two describe the entire interface

// Rust codegen currently pulling in a dep for documents
// so let's just use a union?
structure RunFeatureOutput {
    @required
    data: RunFeatureData
}


// Basic structure to describe the 'stateful-resource' facet
structure StatefulResourceFacet {
    @required
    state: String,

    @required
    transient: Boolean,
}

// Wait. This whole 'types + features' thing is describing a type system. We just hide the entire system
// from the client to simplify things. Clients only use what they need.

// It's debatable as to the value of using an IDL as a mechanism for generating structured data
// Smithy seems to lean more into combining structured data with the interfaces with its `trait` idea
// I'm thinking it'd be better to extract out 'features' via traits

// TODO: operation CheckFeature(feature, resource) <-- checks if a feature is valid against a resource instance