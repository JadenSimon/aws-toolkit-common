$version: "1.0"

namespace aws.toolkits.auth

// TODO: rename all these
use aws.toolkits.samcli#jsonrpcRequest

//-------------- Random stuff --------------//
use smithy.api#trait

/// Describes how a shape should be mapped to an INI file
/// By default, the shape name in snake_case is used
@trait
string profileKey

/// Describes how a shape could be found from environment variables.
/// If this trait does not exist, then one should not make any assumptions
/// about the validity of keys.
@trait
string environmentKey

/// Requires that another shape be present if the targeted shape is used
@trait
@idRef(failWhenMissing: true)
string requires

/// Union version of the above. Requires exactly one shape to be present.
@trait
list requiresOneOf {
    @idRef(failWhenMissing: true)
    member: String // not sure if we need this trait?
}

/// This should strictly be used for information purposes only.
@trait
string defaultValue

/// Constraint trait that restricts the target shape to be a valid path on the filesystem.
@trait
structure filepath {}

//--------------------------//

string ProfileName

/// Represents a configuration to access AWS
/// Secrets are stored on the server
resource Profile {
    identifiers: {
        profileName: ProfileName
    },
    list: ListProfiles,
    resources: [Diagnostic],
}

@enum([
    { name: "SHARED_INI", value: "SharedIni" }
])
string ProfileSource

structure ProfileSummary {
    @required
    name: ProfileName,
    @required
    source: ProfileSource, // do we even need to expose this?
    invalid: Boolean,

}

// Profile operations
@readonly
@jsonrpcRequest(method: "$/aws/auth/profiles/list") // The ugly method name is intended for interop with LSP
operation ListProfiles {
    input: ListProfilesInput,
    output: ListProfilesOutput,
}

structure ListProfilesInput {}

structure ListProfilesOutput {
    @required 
    profiles: Profiles,
}

list Profiles {
    member: ProfileSummary
}

// Misc
@defaultValue("us-east-1") // Should be based off partition, not hard-coded
@profileKey("region")
@environmentKey("AWS_DEFAULT_REGION")
string DefaultRegion

// Static

// Not sure what the best pattern is for these
// It's recommended to play these in the `credentials` file
/// Specifies the AWS access key used as part of the credentials to authenticate the user.
@profileKey("aws_access_key_id")
@environmentKey("AWS_ACCESS_KEY_ID")
string AccessKeyId

/// Specifies the AWS secret key used as part of the credentials to authenticate the user.
@profileKey("aws_secret_access_key")
@environmentKey("AWS_SECRET_ACCESS_KEY")
string SecretAccessKey

/// Specifies an AWS session token used as part of the credentials to authenticate the user. 
/// A session token is required only if you manually specify temporary security credentials.
/// You receive this value as part of the temporary credentials returned by successful requests 
/// to assume a role. 
@profileKey("aws_session_token")
@environmentKey("AWS_SESSION_TOKEN")
string SessionToken

// --------


// SSO
// TODO: there's a smithy bug where it can't handle whitespace + comment in the same line
// All recommended in `config`
string SsoStartUrl
// Obviously should be a region
string SsoRegion 
// Valid AWS Account ID
string SsoAccountId 
// not an ARN, must exist in the specified account
string SsoRoleNam 

// ---------


// Assume role

// All recommended in config

/// You cannot specify both credential_source and source_profile in the same profile.
@enum([
    { name: "Environment", value: "Environment" },
    { name: "Ec2InstanceMetadata", value: "Ec2InstanceMetadata" },
    { name: "EcsContainer", value: "EcsContainer" },
])
@requires(RoleArn)
string CredentialSource

/// Duration in seconds
@range(min: 900, max: 43200)
@profileKey("duration_seconds")
integer SessionDuration

string ExternalId

// This shape is special in that it must be a valid reference to anothr profile
@requires(RoleArn)
string SourceProfile

// @requiresOneOf([CredentialSource, SourceProfile])
string RoleArn

// mfa_serial = arn:aws:iam::123456789012:mfa/my-user-name
// is there an arn pattern match trait?
// can also be a device serial number

string MfaSerial 

// ex:  arn:aws:sts::123456789012:assumed-role/my-role-name/my-role_session_name
// looks like it's limited to certain characters
string RoleSessionName

// This is a path
// Looks like it's an 'assume-role' type of profile
@requires(RoleArn)
string WebIdentityTokenFile

// -----------

// Container

// type as full URI
@environmentKey("AWS_CONTAINER_CREDENTIALS_FULL_URI")
string ContainerCredentialsFullUri 

// type as relative URI
/// Taks priority over the full URI
@environmentKey("AWS_CONTAINER_CREDENTIALS_RELATIVE_URI")
string ContainerCredentialsRelativeUri 

// must match [scheme] [token]
@environmentKey("AWS_CONTAINER_AUTHORIZATION_TOKEN")
string AuthorizationToken 

// ----

// IMDS

// All rcommended in config
@defaultValue("http://169.254.169.254")
//@defaultValue("http://[fd00:ec2::254]")
@environmentKey("AWS_EC2_METADATA_SERVICE_ENDPOINT")
string Ec2MetadataServiceEndpoint

@enum([ { name: "IPv4", value: "IPv4" }, { name: "IPv6", value: "IPv6" } ])
@defaultValue("IPv4")
@environmentKey("AWS_EC2_METADATA_SERVICE_ENDPOINT_MODE")
string Ec2MetadataServiceEndpointMode

// ----

// Process Creds

// Is this a smithy bug: `: Invalid escape found in string: `\-` ?
@filepath
@pattern("^[A-Za-z0-9_.\\\/-]+$")
// The below would be the "escaped" pattern
// @pattern("^\"[A-Za-z0-9\-_.\\\/\s]+\"$")
string ProcessPath

// Parameters are space-delimited
string ProcessParameter

// Best practice is to keep this in `config`
// https://docs.aws.amazon.com/sdkref/latest/guide/feature-process-credentials.html
// bunch of other requirements
/// Path to an executable to load credentials from
string CredentialProcess

// ------


//-------- Features ---------//

// EC2 Metadata (https://docs.aws.amazon.com/sdkref/latest/guide/feature-ec2-instance-metadata.html)

/// This setting specifies the number of total attempts to make before giving up when attempting 
/// to retrieve data from the instance metadata service.
@range(min: 1)
@defaultValue("1") 
@profileKey("metadata_service_num_attempts")
@environmentKey("AWS_METADATA_SERVICE_NUM_ATTEMPTS")
integer MetadataServiceMaxAttempts
// Note that default values should be coerced

/// Specifies the number of seconds before timing out when attempting to retrieve data from 
/// the instance metadata service.
@range(min: 1)
@defaultValue("1")
@environmentKey("AWS_METADATA_SERVICE_TIMEOUT")
integer MetadataServiceTimeout

// S3 Access Points (https://docs.aws.amazon.com/sdkref/latest/guide/feature-s3-access-point.html)
/// This setting controls whether the SDK uses the access point ARN AWS Region to construct the Regional endpoint 
/// for the request. The SDK validates that the ARN AWS Region is served by the same AWS partition as the client's 
/// configured AWS Region to prevent cross-partition calls that most likely will fail. If multiply defined, the 
/// code-configured setting takes precedence, followed by the environment variable setting.
@defaultValue("false")
@environmentKey("AWS_S3_USE_ARN_REGION")
boolean S3UseArnRegion

// S3 multi-region access points (https://docs.aws.amazon.com/sdkref/latest/guide/feature-s3-mrap.html)
/// This setting controls whether the SDK potentially attempts cross-Region requests. If multiple are defined, 
/// the code-configured setting takes precedence, followed by the environment variable setting.
@defaultValue("false")
@profileKey("s3_disable_multiregion_access_points")
@environmentKey("AWS_S3_DISABLE_MULTIREGION_ACCESS_POINTS")
boolean S3MultiRegion


// Region (https://docs.aws.amazon.com/sdkref/latest/guide/feature-region.html)
/// Specifies the default AWS Region to use for AWS requests. This Region is used for SDK service requests 
/// that aren't provided with a specific Region to use.
@environmentKey("AWS_REGION")
string RegionId 
// This should be validated against the partition ('aws-global' is also valid)

// STS (https://docs.aws.amazon.com/sdkref/latest/guide/feature-sts-regionalized-endpoints.html)

// TODO: add descriptions to enum; can we just parse this ??
/// This setting specifies how the SDK or tool determines the AWS service endpoint that it uses to talk to 
/// the AWS Security Token Service (AWS STS).
@enum([ { name: "legacy", value: "legacy" }, { name: "regional", value: "regional" } ])
@defaultValue("regional")
@profileKey("sts_regional_endpoints")
@environmentKey("AWS_STS_REGIONAL_ENDPOINTS")
string StsEndpointType
// Techinically "legacy" by default but all well

// Endpoint Discovery (https://docs.aws.amazon.com/sdkref/latest/guide/feature-endpoint-discovery.html)

/// Enables/disables endpoint discovery for services where endpoint discovery is optional. Endpoint discovery 
/// is required in some services.
@defaultValue("false")
@profileKey("endpoint_discovery_enabled")
@environmentKey("AWS_ENABLE_ENDPOINT_DISCOVERY")
boolean EndpointDiscovery

// General configuration settings (https://docs.aws.amazon.com/sdkref/latest/guide/feature-gen-config.html)

// Note that the below setting is highly non-standard for INI formats...
/// Some AWS services maintain multiple API versions to support backward compatibility. By default, SDK and 
/// AWS CLI operations use the latest available API version. To require a specific API version to use for 
/// your requests, include the api_versions setting in your profile.
blob ApiVersions 
// !!! blob placeholder !!!

/// Specifies the path to a custom certificate bundle (a file with a .pem extension) to use when 
/// establishing SSL/TLS connections.
@filepath
@profileKey("ca_bundle")
@environmentKey("AWS_CA_BUNDLE")
string CertificateAuthorityBundle

/// Specifies whether the SDK or tool attempts to validate command line parameters before sending 
/// them to the AWS service endpoint.
@defaultValue("true")
boolean ParameterValidation


// IMDS client (https://docs.aws.amazon.com/sdkref/latest/guide/feature-imds-client.html)
// NOTE THAT THIS IS NOT ACTUALLY APART OF THE PROFILE CONFIGURATION
// I'M NOT INCLUDING THE REST OF THE ITEMS FOR NOW
/// The number of additional retry attempts for any failed request. 
@range(min: 1)
@defaultValue("3")
@profileKey("retries")
integer MaxAttempts

// Retries (https://docs.aws.amazon.com/sdkref/latest/guide/feature-retry-behavior.html)
// TODO

// Smart Defaults (https://docs.aws.amazon.com/sdkref/latest/guide/feature-smart-config-defaults.html)
// TODO

// --------------------------//

// This is basically just a Document
structure ProfileDetails {}

// What are the valid profile 'types'?
// Static
// Delegated (assume role)
// Federated (SSO)
// ECS ('container') <-- not configurable in ini files
// EC2 (IMDS) <-- partially configurable
// Generated (process)

