package software.amazon.toolkits.telemetry

import com.squareup.kotlinpoet.ClassName
import com.squareup.kotlinpoet.FileSpec
import com.squareup.kotlinpoet.FunSpec
import com.squareup.kotlinpoet.KModifier
import com.squareup.kotlinpoet.MemberName
import com.squareup.kotlinpoet.ParameterSpec
import com.squareup.kotlinpoet.TypeSpec

val PACKAGE_NAME = "software.amazon.toolkits.telemetry"

fun String.filterInvalidCharacters() = this.replace(".", "")

fun String.snakeCaseToCamelCase() =
    this.split('_').map {
        it.toLowerCase().capitalize()
    }.joinToString("")

fun addImports(output: FileSpec.Builder) {
    output.addImport("software.aws.toolkits.jetbrains.services", "telemetry")
}

fun generateTelemetryEnumTypes(output: FileSpec.Builder, items: List<TelemetryMetricType>) {
    items.forEach {
        if (it.allowedValues == null) {
            return@forEach
        }
        val enum = TypeSpec.enumBuilder(it.name)
            .primaryConstructor(
                FunSpec.constructorBuilder()
                    .addParameter("name", String::class)
                    .build()
            )
        it.allowedValues.forEach { enumValue ->
            enum.addEnumConstant(
                enumValue.toString().toUpperCase().filterInvalidCharacters(), TypeSpec.anonymousClassBuilder()
                    .addSuperclassConstructorParameter("%S", enumValue.toString())
                    .build()
            )
        }
        enum.addFunction(FunSpec.builder("toString").addModifiers(KModifier.OVERRIDE).returns(String::class).addStatement("return name").build())
        output.addType(enum.build())
    }
}

//fun generateNamespaces(metrics: List<Metric>): List<TypeSpec.Builder> = metrics.map { it.name.split("_").first().toLowerCase().capitalize() }.map {  }

fun generateRecordFunctions(output: FileSpec.Builder, items: TelemetryDefinition) {
    items
        .metrics
        .sortedBy { it.name }
        .groupBy { it.name.split("_").first().toLowerCase() }
        .forEach { metrics: Map.Entry<String, List<Metric>> ->
            val namespace = TypeSpec.objectBuilder("${metrics.key.capitalize()}Telemetry")
            metrics.value.forEach { metric ->
                val functionBuilder = FunSpec.builder("record${metric.name.split("_")[1].capitalize()}")
                // generate parameters
                val projectParameter = ClassName("com.intellij.openapi.project", "Project").copy(nullable = true)
                val valueParameter = com.squareup.kotlinpoet.DOUBLE
                val additionalParameters = metric.metadata?.map { metadata ->
                    val telemetryMetricType =
                        items.types.find { it.name == metadata.type }
                            ?: throw IllegalStateException("Type ${metadata.type} on ${metric.name} not found in types!")
                    val typeName = if (telemetryMetricType.allowedValues != null) {
                        ClassName(PACKAGE_NAME, telemetryMetricType.name.filterInvalidCharacters())
                    } else {
                        telemetryMetricType.type?.getTypeFromType() ?: com.squareup.kotlinpoet.STRING
                    }.copy(nullable = metadata.required ?: false)
                    ParameterSpec(telemetryMetricType.name.filterInvalidCharacters().toLowerCase(), typeName)
                } ?: listOf()
                functionBuilder
                    .addParameter("project", projectParameter)
                    .addParameter(ParameterSpec.builder("value", valueParameter).defaultValue("1.0").build())
                    .addParameters(additionalParameters)
                // generate body
                val unit = MemberName("software.amazon.awssdk.services.toolkittelemetry.model", "Unit")
                functionBuilder
                    .addStatement("TelemetryService.getInstance().record(project) { ")
                    .addStatement("datum(%S) {", metric.name)
                    .addStatement("unit(%M.${(metric.unit ?: MetricUnit.NONE).name})", unit)
                    .addStatement("value(value)")
                metric.metadata?.forEach {
                    functionBuilder.addStatement("metadata(%S, %L.toString())", it.type, it.type)
                }
                functionBuilder.addStatement("}}")
                namespace.addFunction(functionBuilder.build())
            }
            output.addType(namespace.build())
        }
}

fun main(vararg args: String) {
    val telemetry = parse()
    val output = FileSpec.builder(PACKAGE_NAME, "HelloWorld")
    //val namespaces = generateNamespaces(telemetry.metrics)
    addImports(output)
    generateTelemetryEnumTypes(output, telemetry.types)
    generateRecordFunctions(output, telemetry)
    output.build().writeTo(System.out)
}
