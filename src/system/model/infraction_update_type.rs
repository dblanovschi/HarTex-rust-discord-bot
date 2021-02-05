#[derive(Clone)]
crate enum InfractionUpdateType {
    Reason {
        new_reason: String
    }
}
