+++
id = "I-012"
title = "identity canonicalized under the tardigrada namespace"
status = "open"
created = "2026-11-16"
owner = "devin"
priority = "med"
labels = ["identity"]
+++

cuora now resolves the shared tardigrada age identity as canonical: keyring entry tardigrada/age-identity (store writes only tardigrada; load falls back tardigrada → cuora → arca) and keyfile <config>/tardigrada/identity.txt (legacy cuora/arca paths still resolve). Shipped in cuora 1.0.2.
