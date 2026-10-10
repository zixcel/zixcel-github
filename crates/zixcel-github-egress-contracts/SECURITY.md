# Security boundary

An authorized request is a one-way validated value. Its URL and body are not
available until the transport consumes it. Bodies are zeroized on drop and the
types deliberately omit `Debug`, `Clone`, `Serialize` and `Deserialize`.

Redirect, DNS, proxy and TLS behavior belong to the concrete Crowsi-approved
transport. A redirect must be returned for a new policy decision; it must not
be followed implicitly with authorization request material.

## Private vulnerability reporting

Report vulnerabilities through this repository's GitHub private vulnerability reporting form. Do not put credentials, personal or customer data, or production certificate material in public issues or pull requests.
