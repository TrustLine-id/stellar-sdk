use soroban_sdk::{contractevent, Address};

#[contractevent(topics = ["fw", "tgt"], data_format = "single-value")]
pub struct FirewallTargetUpdated {
    pub new_target: Address,
}

#[contractevent(topics = ["fw", "own"], data_format = "vec")]
pub struct FirewallOwnerUpdated {
    pub old_owner: Address,
    pub new_owner: Address,
}

#[contractevent(topics = ["fw", "opr"], data_format = "vec")]
pub struct FirewallOperatorUpdated {
    pub account: Address,
    pub is_operator: bool,
}

#[contractevent(topics = ["fw", "pub"], data_format = "single-value")]
pub struct FirewallPublicForwardUpdated {
    pub enabled: bool,
}
