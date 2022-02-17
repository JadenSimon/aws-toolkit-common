$version: "1.0"

namespace aws.toolkits.core

// Just some notes on IRI structure

structure Authority {
    /// In the context of the toolkits, `userinfo` may be used to identify how the server should authenticate.
    /// This should NEVER contain any real credentials. Strings that contain a colon (`:`) will be rejected.
    /// Userinfo should always be followed by an at symbol (`@`).
    /// 
    /// If present, this field should not affect comparisons between other IRIs when used as a resource locator.
    userinfo: String, 

    @required
    host: String,

    port: Integer,
}

structure IRI2 {
    @required
    scheme: String,

    // Preceded by two slashes (`//`)
    authority: Authority,
}