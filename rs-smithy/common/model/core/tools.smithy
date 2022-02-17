$version: "1.0"

namespace aws.toolkits.core

// This 'module' is largely for CLI tooling that the backend runs
// We want to be somewhat transparent about this, so we expose a few interfaces
// Could be big enough to break into a separate namespace

string ToolId
// The ideal is that this would strictly be SemVer 
string Version 

/// An 'abstract' tool that includes all versions
resource Tool {
    identifiers: {
        id: ToolId,
    },
    resources: [VersionedTool],
}

// Big note! We should never expose any interface to execute arbitrary binaries (like, ever)
resource VersionedTool {
    identifiers: {
        id: ToolId,
        version: Version,
        // There could be more identifiers here such as architecture, operating system, etc.
        // I think it's best to just hide that stuff, at least in the interface
    }
}

// Maybe this should go somewhere else? It's just the host machine's environment.
structure Environment {

}

@readonly
operation ListTools {
    input: ListToolsInput,
    output: ListToolsOutput,
}

structure ListToolsInput {}
structure ListToolsOutput {}

list InstalledVersions {
    member: Version,
}

// naming consideration: `details` implies that this is fully-descriptive, `summary` implies the opposite
structure ToolSummary {
    @required
    id: ToolId,

    // Maybe just include all info here
    // Including only the version means another call
    @required
    installed: InstalledVersions,

    // This is like a version override; by default we'd want to use the latest
    selectedVersion: Version,

    // latestVersion ?
    // sources ?
    // who knows!
}


@readonly
operation ListVersions {
    input: ListVersionsInput,
    output: ListVersionsOutput,
}

structure ListVersionsInput {}
structure ListVersionsOutput {}

structure VersionedToolDetails {
    @required
    id: ToolId,

    @required
    version: Version,

    // Absence means it isn't installed
    // A union type could make sense if codegen is consistent
    installationPath: IRI,
}

// Smithy doesn't seem to have a way to 'condense' IDs into a URI?
// Either way this ID can be represented as `${ToolId}/${Version}/${Unid}`
// Where `Unid` is generated at runtime
string SpawnedToolId

/// 'spawned' tools represent specific instances of tool executions as opposed to metadata about it.
resource SpawnedTool {
    identifiers: {
        id: SpawnedToolId,
    },
    list: ListSpawnedTools,
    operations: [ForwardStreams],
}

list SpawnedTools {
    member: SpawnedToolSummary,
}

// Need to make a macro or something...
@readonly
operation ListSpawnedTools {
    input: ListSpawnedToolsInput,
    output: ListSpawnedToolsOutput,
}

structure ListSpawnedToolsInput {}
structure ListSpawnedToolsOutput {
    @required
    tools: SpawnedTools,
}


// TODO: errors!
operation ForwardStreams {
    input: ForwardStreamsInput,
    output: ForwardStreamsOutput,
}

structure Message {
    message: String,
    // message: Blob,
}

/// Using a union means we don't need to manage multiple sockets
@streaming
union StdioStream {
    stdin: Message,
    stdout: Message,
    stderr: Message,
    close: Message,
}

structure ForwardStreamsInput {
    @required
    id: SpawnedToolId,

    @required
    stream: StdioStream,
}

structure ForwardStreamsOutput {
    @required
    stream: StdioStream,
}

structure SpawnedToolSummary {
    @required
    id: SpawnedToolId,

    /// A 'forwarded' tool means stdio is being proxied to a client.
    /// One can think of this as a client taking 'ownership' of the tool as the server
    /// will not exercise control except when fatal errors occur.
    @required
    forwarded: Boolean,

    // TODO: add way to just listen to a tool's output rather than taking the streams

}