$version: "1.0"

namespace aws.toolkits.core

use smithy.api#trait

// How would a service like Consolas fit in this model?
// They fit pretty well under some sort of 'language server' classification.
// So Consolas would be a Service, just not a 'displayable' one

// What about things like user settings?
// These would be something a type 'contributes' (not unlike VS Code's `package.json`)
// The frontend would be in charge of managing settings, responding to backend requests as needed

// For things that don't neatly 'fit' we'd have to define new types, e.g. webviews

// Modeling things like this will be inherently lossy. There doesn't seem to be a way around de-duplication across
// multiple IDEs without abstracting away detail. 
//
// Wow this is a tough problem. I think it's still worth pursuing though?
// The problem boils down to picking the right level of abstraction. Too little and it isn't worth doing.
// Too much and it becomes difficult to implement things that don't fit nicely within the abstraction.
// Many of the common operations on resources are already well defined by API models, but we lack
// a semantic description for these shapes.

/// Purely a design-time trait to describe relationships between types
@trait
structure type {}

/// Describes what other resources that could logically be placed as 'children' to an instance of this type
@trait
list contains {
    member: ResourceType
}

// Describes a few different basic types
// By default, all resources have a `Unit` type 

/// This specifically relates to an AWS region
@type 
structure Region {}

/// A service is, fundamentally, a resource that provides features
@type
structure Service {}