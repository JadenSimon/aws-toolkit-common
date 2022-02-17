$version: "1.0"

namespace aws.toolkits.auth

// TODO: rename all these
use aws.toolkits.samcli#jsonrpc

@jsonrpc(version: "2.0")
service Credentials {
    version: "0.0.1",
    resources: [Profile],
}
