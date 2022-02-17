$version: "1.0"

namespace aws.toolkits.samcli

// Defines annotations for describing a JSON-RPC based protocol

use smithy.api#trait

// Only 2.0 allowed for now
string JsonRpcVersion

@protocolDefinition
@trait(selector: "service")
structure jsonrpc {
    @required
    version: JsonRpcVersion
}

/// Requests should always have a unique id
/// Note that the protocol is largely bi-directional;
/// While there is often an explicit 'server', either side of the connection may do both
/// TODO: support 'batch' reqeusts by checking for list inputs?
@trait(selector: "operation")
structure jsonrpcRequest {
    @required
    method: String,
}

/// Notifications should never return a response
/// They do not need to have an id
/// They also can be bi-directional
@trait(selector: "operation")
structure jsonrpcNotification {
    @required
    method: String,
}

/// Support only structures for now
@trait(selector: "structure")
structure jsonrpcError {
    @required
    code: Integer,
    @required
    message: String,
}
