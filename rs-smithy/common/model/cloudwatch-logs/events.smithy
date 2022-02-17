$version: "1.0"

namespace aws.toolkits.cloudwatchlogs

// There's litle reason to not use the pre-existing definitions here
// Using the same shapes helps preserve semantics 
use com.amazonaws.cloudwatchlogs#LogGroupName
use com.amazonaws.cloudwatchlogs#LogStreamName
use com.amazonaws.cloudwatchlogs#OutputLogEvents

use aws.toolkits.samcli#jsonrpc
use aws.toolkits.samcli#jsonrpcRequest

// PoC for an ephemeral 'scrollable' backed by pagination
// We essentially turn each item in a API result into a 'line' in the document
//
// The fact that log events are viewed in pages is a technical limitation which is what 
// this abstraction aims to solve

@jsonrpc(version: "2.0")
service ToolkitsCloudwatchLogs {
    version: "2021-01-01",
    resources: [LogStreamViewer],
}

// ---- Stream Viewer ---- //

string LogStreamViewerId

@enum([
    { name: "UP", value: "up", },
    { name: "DOWN", value: "down" }
])
string StartDirection

/// Stateful entity that can be used to view CloudWatch log events as a pseudo-document
resource LogStreamViewer {
    identifiers: {
        logStreamViewerId: LogStreamViewerId,
    },
    create: CreateLogStreamViewer,
    read: GetWindow,
    // TODO: delete
    // `delete` becomes necessary to clean-up any resources on the backend
    operations: [ScrollTo],
}

operation CreateLogStreamViewer {
    input: CreateLogStreamViewerInput,
    output: CreateLogStreamViewerOutput,
}

structure CreateLogStreamViewerInput {
    @required
    logGroupName: LogGroupName,
    @required
    logStreamName: LogStreamName,

    /// Changes how events are streamed in. This is equivalent to "head" or "tail"
    direction: StartDirection,
}

structure CreateLogStreamViewerOutput {
    @required
    logStreamViewerId: LogStreamViewerId,
}

// ----------------------- //

// -------- State -------- //

structure ViewerState {
    @required
    /// Represents the current position in the stream by number of log events after any filtering
    cursor: Integer,
    /// Size of the current page (not including buffers).
    pageSize: Integer,
}

/// Gets the viewer's currently 'viewed' log events
@readonly
operation GetWindow {
    input: GetWindowInput,
    output: GetWindowOutput,
}

structure GetWindowInput {
    @required
    logStreamViewerId: LogStreamViewerId,
}

structure GetWindowOutput {
    @required
    /// Represents the current position in the stream by number of log events after any filtering
    cursor: Integer,
    
    @required
    events: OutputLogEvents
}


// ------ Rendering ------ //

/// Tells the viewer to 'scroll' to the designated position in the stream
operation ScrollTo {
    input: ScrollToInput,
    output: GetWindowOutput,
}

structure ScrollToInput {
    @required
    logStreamViewerId: LogStreamViewerId,

    @required
    /// Log event index to scroll to. 
    position: Integer,
}
