$version: "1.0"

namespace aws.toolkits.samcli

use aws.toolkits.flow#configType
use aws.toolkits.flow#FlowSchemaElement

// Note that this model is for a high-level abstraction over the SAM CLI command
// The generated model is a reference point

// There are two ideas at play here): 
// * A stateless `Question` template
// * A stateful `Question` object
//
// The first case makes the client responsible for handling all state; the server only provides basic
// information about how to ask those questions. The client then needs to interpret the information to
// craft a proper UI, potentially storing state up to the point of when it's needed.
//
// The second is a stateful entity that the server tracks. It's essentially a combination of the state
// and template. This hides many details from the client since they need to know much less about
// how questions change in response to changing state. Clients become extremely thin, needing to only
// know how to render UI elements depending on the type of 'question' as well as know when (and what)
// to respond to the server when things change.
//
// This is not too unlike how VS Code is built - there is a clear demarcation between the UI (the 'window')
// and the working environment (the 'workspace'). And as far as I can tell, this has worked quite well for
// them. That's why I'm in favor of the 2nd option.
//

// Dummy operation for prototyping
operation CreatePipelineStage {
    input: CreatePipelineStageInput,
    output: PipelineStage,
}

structure CreatePipelineStageInput {}


// scratch space for writing out SAM pipeline wizard UI template
// the CI/CD pipelines can be split up into arbitrary stages, so let's model it like that
// the 'prelude' would need the following:
// 1. auth (varies based off template) <-- this part needs to be extensible
// 2. git branch
// 3. SAM template
// now we can add stages
// a stage would have:
// 1. stage name (e.g. testing)
// 2. stack name (perhaps this targets the template?)
// 3. pipeline executation role (can pre-fill)
// 4. CFN execution role to deploy (can pre-fill)
// 5. S3 bucket for artifacts (can pre-fill)
// 6. ECR repo (can pre-fill) [optional, but not sure what the criteria is]
// 7. region (can pre-fill)
// `sam bootstrap` should be done then we can create the template
// `sam bootstrap` just creates 1 stage at a time (good for us)

// this is basically the `auth` step mentioned above
union PipelinePlatform {
    data: String,
}

// These are all placeholdes (maybe besides stagename)
string GitBranch
string SamTemplate 
string StageName
// S3 obviously
string Bucket 
string IamUser
// TODO: figure this out
string IamRole 
// ECR
string ImageRepository

// There are a few other settings, but they apply to most SAM CLI commands:
// * profile
// * region
// * config file (`samconfig.toml`)
// * config env (TOML file section)

structure PipelineTargetConfiguration {
    @required 
    //@configType(PipelinePlatform)
    platform: FlowSchemaElement, 

    @required 
    //@configType(GitBranch)
    branch: FlowSchemaElement, 

    @required 
    //@configType(SamTemplate)
    template: FlowSchemaElement, 
}

// By definition, this is a subtype of 'FlowSchema', though Smithy doesn't really have any idea of inheritance
structure PipelineStage {
    @required 
    //@configType(StageName)
    name: FlowSchemaElement, 

    @required 
    //@configType(IamUser)
    pipelineUser: FlowSchemaElement, 

    @required 
    //@configType(IamRole)
    cloudformationExecutionRole: FlowSchemaElement, 

    @required 
    //@configType(IamRole)
    pipelineExecutionRole: FlowSchemaElement, 

    @required 
    //@configType(Bucket)
    artifactBucket: FlowSchemaElement, 

    @required 
    //@configType(ImageRepository)
    /// Image repository is only needed for image lambdas
    imageRepository: FlowSchemaElement, 

    // not sure what the point of this one is?
    // omitted since we will always prompt
    // confirmChangeset: FlowSchemaElement,
}


// Stub for now. Should validate?
//structure QuestionResponseOutput {
    /// While in this prototype the state ID is always valid, the client should never assume that results from 
    /// previous identifiers are still valid. This is because some questions could potentially cause intermediate 
    /// side-effects, thus changing the context.
    //@required
    //nextState: StateId,

    /// Signals that there are no more questions to answer.
    //completed: Boolean,
//}


// After answering a question, the current set of questions is 'stale'; the client must re-fetch.


