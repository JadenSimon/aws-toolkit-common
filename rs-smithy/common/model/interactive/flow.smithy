$version: "1.0"

namespace aws.toolkits.interactive

// An abstract model that describes interactive flows.
// These flows are modeled as acyclic dependency graphs where vertices
// represent sub-graphs and edges represent state transitions (i.e. user responses).
//
// The nature of this model implies that the consumers has no knowledge of the full graph,
// so clients must use the provided edges to determine if they should fetch a new sub-graph.
// An invariant placed upon sub-graphs is that they must be directly connected to eachother
// for an edge to exist.
//
// From a UI perspective, edges can be thought of as user input while each vertex is a permutation
// of the current state. Thus an edge could be mapped 1:1 with an input element.
//
// Mathematically, this can be thought of as a Moore machine, albeit one with complex state and transitions.


// Other notes:
// New vertices can be derived from existing ones. Whether or not they are connected to a given graph
// is how a state transition is determined to be 'valid'. So an individual abstract edge could be thought 
// of as the set of all concrete (E, V) pairs. And so the problem becomes such: given a current vertex V
// and a new vertex V', does there exist an edge (V, V')? If so, that edge is a valid user input.

service Flow {
    version: "0.0.1",
    resources: [Graph],
}

string GraphId 
string VertexId
/// Unique identifier associated with a specific edge. 
string EdgeId

/// A single graph may describe an entire flow. 
/// It is implementation-dependent as to whether the full graph can be dervived at any given point.
resource Graph {
    identifiers: {
        graphId: GraphId,
    },
}

/// A vertex is an abstract state within a graph. Graphs will always contain at least 1 vertex that
/// represents the initial state.
resource Vertex {
    identifiers: {
        vertexId: VertexId,
        graphId: GraphId,
    },
}

/// As edges describe state transitions, they can be used as a mechanism for creating UI elements.
resource Edge {
    identifiers: {
        edgeId: EdgeId,
        graphId: GraphId,
    },
}

//-------------- Operations --------------//

// TODO: add errors

@readonly
operation ListGraphs {
    input: ListGraphsInput,
    output: ListGraphsOutput,
}

/// Note that only concrete vertices are enumerated.
@readonly
operation ListVertices {
    input: ListVerticesInput,
    output: ListVerticesOutput,
}

@readonly
operation ListEdges {
    input: ListEdgesInput,
    output: ListEdgesOutput,
}

//-------------- Structures --------------//


// Graph
structure GraphSummary {
    @required
    graphId: GraphId,
}

list Graphs {
    member: GraphSummary
}

structure ListGraphsInput {}

structure ListGraphsOutput {
    graphs: Graphs,
}


// Vertex
structure VertexSummary {
    @required
    vertexId: VertexId,
}

list Vertices {
    member: VertexSummary
}

structure ListVerticesInput {
    @required
    graphId: GraphId,
}

structure ListVerticesOutput {
    vertices: Vertices,
}


// Edge
structure EdgeSummary {
    @required
    edgeId: EdgeId,

    /// The 'tail' is always the first vertex in an ordered pair of vertices
    /// While edges themselves may not necessarily be directed, from the client's
    /// perspective they are.
    @required
    tail: VertexId,

    /// This field is not present if the edge is connected to an abstract vertex.
    head: VertexId,
}

list Edges {
    member: EdgeSummary
}

structure ListEdgesInput {
    @required
    graphId: GraphId,
}

structure ListEdgesOutput {
    edges: Edges,
}