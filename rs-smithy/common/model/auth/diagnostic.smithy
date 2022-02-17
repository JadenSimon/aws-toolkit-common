$version: "1.0"

namespace aws.toolkits.auth

// TODO: rename all these
use aws.toolkits.samcli#jsonrpcRequest

/// While most wouldn't consider this to be a 'Resource', it logically makes sense when
/// one considers the relationship between a profile and any issues with it
resource Diagnostic {
    identifiers: {
        profileName: ProfileName,
        id: String, // Unique ID associated with the current document state
    },
    list: ListDiagnostics,
}

structure DiagnosticSummary {
    @required
    id: String,
    @required
    message: String,
}

@readonly
@jsonrpcRequest(method: "$/aws/auth/profiles/diagnostics/list") 
operation ListDiagnostics {
    input: ListDiagnosticsInput,
    output: ListDiagnosticsOutput,
}

structure ListDiagnosticsInput {
    @required
    profileName: ProfileName,
}

structure ListDiagnosticsOutput {
    @required 
    diagnostics: Diagnostics,
}

list Diagnostics {
    member: DiagnosticSummary
}