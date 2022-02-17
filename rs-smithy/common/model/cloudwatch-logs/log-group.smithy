$version: "1.0"

namespace aws.toolkits.cloudwatchlogs

// Another day, another model...
// This time we're taking the information hiding approach

string LogGroupName
string LogGroupArn

resource LogGroup {
    identifiers: {
        name: LogGroupName
    },
    list: DescribeLogGroups,

}

@readonly
operation DescribeLogGroups {
    input: DescribeLogGroupsInput,
    output: DescribeLogGroupsOutput,
}

structure DescribeLogGroupsInput {}
structure DescribeLogGroupsOutput {
    @required
    logGroups: LogGroups,
}

list LogGroups {
    member: LogGroupSummary,
}

/// Since this is the 'information-hiding' approach
/// This structure renames things to directly map to UI content
structure LogGroupSummary {
    @required
    name: LogGroupName,

    @required
    arn: LogGroupArn,

    @required
    /// An opaque token that signifies the current 'state' of the resource
    /// Any operations on the resource must provide the token
    token: String,

    /// Long-form information, possibly useful for tooltips
    description: String,

    /// Additional info but not super important
    detail: String,
}


// Just some more notes:
// How 'thin' should clients be? The less power offered to them, the harder it will be to create idiomatic experiences.
// At the same time, having no common backend means any experience in one toolkit has to basically be re-implemented in another.
// I suppose the 'balance' here could be offering a way to short-circuit the backend abstractions and allow direct access to
// the AWS APIs. But this wouldn't have '1st class' support; the backend would just act as a proxy 