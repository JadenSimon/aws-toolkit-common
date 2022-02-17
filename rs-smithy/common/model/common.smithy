$version: "1.0"

namespace aws.toolkits.samcli

// All shapes in here need validation (including plain String)

use smithy.api#trait

string Uri

map TemplateParameters {
    key: String,
    value: String,
}

list ProcessArguments {
    member: String
}

// Protocols
// These will just be a JSON <--> CLI protocol for now...
@protocolDefinition
@trait(selector: "service")
structure stdio {}

@trait(selector: "operation")
structure process {
    /// Additional arguments to prepend to built command
    arguments: ProcessArguments
    // How should this protoocl even look?
    // It's not exactly complicated, but it is different since things can be streamed
    // I think realistically, it would open up streams on the client as well
}

@trait(selector: "structure > member")
structure processOption {
    @required
    switch: String,
    /// Allow more than one switch. This only works for list types.
    allowMultiple: Boolean,
}

@trait(selector: "structure > member")
structure processArgument {
    order: Integer
}