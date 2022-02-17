$version: "1.0"

namespace aws.toolkits.samcli

list ContainerEnvVar {
    member: String
}


list BuildImage {
    member: String
}

structure BuildInput {
    /// The environment name specifying the default parameter values in the configuration file to use. Its default value is 'default'. For more information about configuration files, see: https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/serverless-sam-cli-config.html.
    @processOption(switch: "--config-env")
    configEnv: String,

    /// The path and file name of the configuration file containing default parameter values to use. Its default value is 'samconfig.toml' in project directory. For more information about configuration files, see: https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/serverless-sam-cli-config.html.
    @processOption(switch: "--config-file")
    configFile: String,

    /// If your functions depend on packages that have natively compiled dependencies, use this flag to build your function inside an AWS Lambda-like Docker container
    @processOption(switch: "-u")
    useContainer: Boolean,

    /// Input environment variables through command line to pass into build containers, you can either input function specific format (FuncName.VarName=Value) or global format (VarName=Value). e.g., sam build --use-container --container-env-var Func1.VAR1=value1 --container-env-var VAR2=value2
    @processOption(switch: "-e", allowMultiple: true)
    containerEnvVar: ContainerEnvVar,

    /// Path to environment variable json file (e.g., env_vars.json) to pass into build containers
    @processOption(switch: "-ef")
    containerEnvVarFile: Uri,

    /// Container image URIs for building functions/layers. You can specify for all functions/layers with just the image URI (--build-image public.ecr.aws/sam/build-nodejs14.x:latest). You can specify for each individual function with (--build-image FunctionLogicalID=public.ecr.aws/sam/build-nodejs14.x:latest). A combination of the two can be used. If a function does not have build image specified or an image URI for all functions, the default SAM CLI build images will be used.
    @processOption(switch: "-bi")
    buildImage: BuildImage,

    /// Enabled parallel builds. Use this flag to build your AWS SAM template's functions and layers in parallel. By default the functions and layers are built in sequence
    @processOption(switch: "-p")
    parallel: Boolean,

    /// Path to a folder where the built artifacts will be stored. This directory will be first removed before starting a build.
    @processOption(switch: "-b")
    buildDir: Uri,

    /// The folder where the cache artifacts will be stored when --cached is specified. The default cache directory is .aws-sam/cache
    @processOption(switch: "-cd")
    cacheDir: Uri,

    /// Resolve relative paths to function's source code with respect to this folder. Use this if SAM template and your source code are not in same enclosing folder. By default, relative paths are resolved with respect to the SAM template's location
    @processOption(switch: "-s")
    baseDir: Uri,

    /// Path to a custom dependency manifest (e.g., package.json) to use instead of the default one
    @processOption(switch: "-m")
    manifest: Uri,

    /// Enable cached builds. Use this flag to reuse build artifacts that have not changed from previous builds. AWS SAM evaluates whether you have made any changes to files in your project directory. 
    /// 
    /// Note: AWS SAM does not evaluate whether changes have been made to third party modules that your project depends on, where you have not provided a specific version. For example, if your Python function includes a requirements.txt file with the following entry requests=1.x and the latest request module version changes from 1.1 to 1.2, SAM will not pull the latest version until you run a non-cached build.
    @processOption(switch: "-c")
    cached: Boolean,

    /// AWS SAM template file.
    @processOption(switch: "-t")
    templateFile: Uri,

    /// Optional. A string that contains AWS CloudFormation parameter overrides encoded as key=value pairs.For example, 'ParameterKey=KeyPairName,ParameterValue=MyKey ParameterKey=InstanceType,ParameterValue=t1.micro' or KeyPairName=MyKey InstanceType=t1.micro
    @processOption(switch: "--parameter-overrides")
    parameterOverrides: TemplateParameters,

    /// Specify whether CLI should skip pulling down the latest Docker image for Lambda runtime.
    @processOption(switch: "--skip-pull-image")
    skipPullImage: Boolean,

    /// Specifies the name or id of an existing docker network to lambda docker containers should connect to, along with the default bridge network. If not specified, the Lambda containers will only connect to the default bridge docker network.
    @processOption(switch: "--docker-network")
    dockerNetwork: String,

    /// Should beta features be enabled.
    @processOption(switch: "--beta-features")
    betaFeatures: Boolean,

    /// Turn on debug logging to print debug message generated by SAM CLI and display timestamps.
    @processOption(switch: "--debug")
    debug: Boolean,

    /// Select a specific profile from your credential file to get AWS credentials.
    @processOption(switch: "--profile")
    profile: String,

    /// Set the AWS Region of the service (e.g. us-east-1).
    @processOption(switch: "--region")
    region: String,

    resourceLogicalId: String,
}

/// /// Use this command to build your AWS Lambda Functions source code to generate artifacts that target AWS Lambda's
/// execution environment.
/// 
/// 
/// Supported Resource Types
/// ------------------------
/// 1. AWS::Serverless::Function
/// 
/// 2. AWS::Lambda::Function
/// 
/// 
/// Supported Runtimes
/// ------------------
/// 1. Python 2.7, 3.6, 3.7, 3.8 3.9 using PIP
/// 
/// 2. Nodejs 14.x, 12.x, 10.x, 8.10, 6.10 using NPM
/// 
/// 3. Ruby 2.5 using Bundler
/// 
/// 4. Java 8, Java 11 using Gradle and Maven
/// 
/// 5. Dotnetcore2.0 and 2.1 using Dotnet CLI (without --use-container flag)
/// 
/// 6. Go 1.x using Go Modules (without --use-container flag)
/// 
/// 
/// Examples
/// --------
/// To use this command, update your SAM template to specify the path
/// to your function's source code in the resource's Code or CodeUri property.
/// 
/// To build on your workstation, run this command in folder containing
/// SAM template. Built artifacts will be written to .aws-sam/build folder
/// $ sam build
/// 
/// 
/// To build inside a AWS Lambda like Docker container
/// $ sam build --use-container
/// 
/// To build with inline environment variables passed inside build containers
/// $ sam build --use-container --container-env-var Function.ENV_VAR=value --container-env-var GLOBAL_ENV_VAR=value
/// 
/// To build with environment variables file passd inside build containers
/// $ sam build --use-container --container-env-var-file env.json
/// 
/// To build & run your functions locally
/// $ sam build && sam local invoke
/// 
/// To build and package for deployment
/// $ sam build && sam package --s3-bucket <bucketname>
/// 
/// To build only an individual resource (function or layer) located in the SAM
/// template. Downstream SAM package and deploy will deploy only this resource
/// $ sam build MyFunction
@process
operation RunBuild {
    input: BuildInput
}
