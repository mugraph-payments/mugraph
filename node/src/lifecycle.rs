use mugraph_core::types::{
    TransferChainState, TransferCreditState, TransferSettlementState, TransferStatusPayload,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceLaneState {
    Requested,
    Submitted,
    Confirming,
    Confirmed,
    Invalidated,
}

impl SourceLaneState {
    pub const fn as_status_str(self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Submitted => "submitted",
            Self::Confirming => "confirming",
            Self::Confirmed => "confirmed",
            Self::Invalidated => "invalidated",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestinationLaneState {
    NoticeReceived,
    ChainObserved,
    CreditEligible,
    Credited,
    Invalidated,
}

impl DestinationLaneState {
    pub const fn as_status_str(self) -> &'static str {
        match self {
            Self::NoticeReceived => "notice_received",
            Self::ChainObserved => "chain_observed",
            Self::CreditEligible => "credit_eligible",
            Self::Credited => "credited",
            Self::Invalidated => "invalidated",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleEvent<'a> {
    SourceSubmitted,
    DestinationNoticeReceived,
    ChainObserved {
        tx_hash: &'a str,
        confirmations: u32,
        confirmed: bool,
    },
    DestinationCredited,
    ChainInvalidated,
    AckReceived,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferLifecycle {
    pub source: SourceLaneState,
    pub destination: DestinationLaneState,
    pub settlement: TransferSettlementState,
    pub chain: TransferChainState,
    pub credit: TransferCreditState,
    pub tx_hash: Option<String>,
    pub confirmations_observed: u32,
}

impl TransferLifecycle {
    pub fn new() -> Self {
        Self {
            source: SourceLaneState::Requested,
            destination: DestinationLaneState::NoticeReceived,
            settlement: TransferSettlementState::NotSubmitted,
            chain: TransferChainState::Unknown,
            credit: TransferCreditState::None,
            tx_hash: None,
            confirmations_observed: 0,
        }
    }

    pub fn apply(&mut self, event: LifecycleEvent<'_>) {
        match event {
            LifecycleEvent::SourceSubmitted => {
                if self.source == SourceLaneState::Requested {
                    self.source = SourceLaneState::Submitted;
                    self.settlement = TransferSettlementState::Submitted;
                    self.chain = TransferChainState::Submitted;
                }
            }
            LifecycleEvent::DestinationNoticeReceived => {
                // Notice delivery is idempotent and must never regress state.
            }
            LifecycleEvent::ChainObserved {
                tx_hash,
                confirmations,
                confirmed,
            } => {
                if self.chain == TransferChainState::Invalidated {
                    return;
                }

                self.tx_hash = Some(tx_hash.to_string());
                self.confirmations_observed = confirmations;

                if confirmed {
                    self.source = SourceLaneState::Confirmed;
                    self.destination = match self.destination {
                        DestinationLaneState::Credited => DestinationLaneState::Credited,
                        DestinationLaneState::Invalidated => DestinationLaneState::Invalidated,
                        _ => DestinationLaneState::CreditEligible,
                    };
                    self.chain = TransferChainState::Confirmed;
                    self.settlement = TransferSettlementState::Confirmed;
                    if self.credit != TransferCreditState::Credited {
                        self.credit = TransferCreditState::Eligible;
                    }
                } else if self.source != SourceLaneState::Confirmed {
                    self.source = SourceLaneState::Confirming;
                    if self.destination == DestinationLaneState::NoticeReceived {
                        self.destination = DestinationLaneState::ChainObserved;
                    }
                    self.chain = TransferChainState::Confirming;
                    self.settlement = TransferSettlementState::Confirming;
                }
            }
            LifecycleEvent::DestinationCredited => {
                if self.source == SourceLaneState::Confirmed
                    && self.destination != DestinationLaneState::Invalidated
                {
                    self.destination = DestinationLaneState::Credited;
                    self.credit = TransferCreditState::Credited;
                }
            }
            LifecycleEvent::ChainInvalidated => {
                self.source = SourceLaneState::Invalidated;
                self.destination = DestinationLaneState::Invalidated;
                self.chain = TransferChainState::Invalidated;
                self.settlement = TransferSettlementState::Invalidated;
                self.credit = if self.credit == TransferCreditState::Credited {
                    TransferCreditState::Reversed
                } else {
                    TransferCreditState::None
                };
            }
            LifecycleEvent::AckReceived => {}
        }
    }

    pub fn to_status_payload(&self, updated_at: String) -> TransferStatusPayload {
        TransferStatusPayload {
            source_state: self.source.as_status_str().to_string(),
            destination_state: self.destination.as_status_str().to_string(),
            settlement_state: self.settlement.clone(),
            chain_state: self.chain.clone(),
            credit_state: self.credit.clone(),
            tx_hash: self.tx_hash.clone(),
            confirmations_observed: self.confirmations_observed,
            updated_at,
        }
    }
}

impl Default for TransferLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn lifecycle_source_lane_advances_to_confirmed() {
        let mut lifecycle = TransferLifecycle::new();

        lifecycle.apply(LifecycleEvent::SourceSubmitted);
        lifecycle.apply(LifecycleEvent::ChainObserved {
            tx_hash: "abc",
            confirmations: 3,
            confirmed: false,
        });
        lifecycle.apply(LifecycleEvent::ChainObserved {
            tx_hash: "abc",
            confirmations: 12,
            confirmed: true,
        });

        assert_eq!(lifecycle.source, SourceLaneState::Confirmed);
        assert_eq!(lifecycle.chain, TransferChainState::Confirmed);
        assert_eq!(lifecycle.settlement, TransferSettlementState::Confirmed);
    }

    #[test]
    fn lifecycle_destination_lane_advances_to_credited() {
        let mut lifecycle = TransferLifecycle::new();

        lifecycle.apply(LifecycleEvent::SourceSubmitted);
        lifecycle.apply(LifecycleEvent::ChainObserved {
            tx_hash: "abc",
            confirmations: 12,
            confirmed: true,
        });
        lifecycle.apply(LifecycleEvent::DestinationCredited);

        assert_eq!(lifecycle.destination, DestinationLaneState::Credited);
        assert_eq!(lifecycle.credit, TransferCreditState::Credited);
    }

    #[test]
    fn stale_ack_does_not_regress_confirmed_outcome() {
        let mut lifecycle = TransferLifecycle::new();

        lifecycle.apply(LifecycleEvent::SourceSubmitted);
        lifecycle.apply(LifecycleEvent::ChainObserved {
            tx_hash: "abc",
            confirmations: 12,
            confirmed: true,
        });
        let before = lifecycle.clone();

        lifecycle.apply(LifecycleEvent::AckReceived);

        assert_eq!(lifecycle, before);
    }

    #[test]
    fn duplicate_notice_does_not_regress_credited_outcome() {
        let mut lifecycle = TransferLifecycle::new();

        lifecycle.apply(LifecycleEvent::SourceSubmitted);
        lifecycle.apply(LifecycleEvent::ChainObserved {
            tx_hash: "abc",
            confirmations: 12,
            confirmed: true,
        });
        lifecycle.apply(LifecycleEvent::DestinationCredited);
        let before = lifecycle.clone();

        lifecycle.apply(LifecycleEvent::DestinationNoticeReceived);

        assert_eq!(lifecycle, before);
    }

    #[test]
    fn deep_reorg_forces_invalidated_path_deterministically() {
        let mut lifecycle = TransferLifecycle::new();

        lifecycle.apply(LifecycleEvent::SourceSubmitted);
        lifecycle.apply(LifecycleEvent::ChainObserved {
            tx_hash: "abc",
            confirmations: 12,
            confirmed: true,
        });
        lifecycle.apply(LifecycleEvent::DestinationCredited);

        lifecycle.apply(LifecycleEvent::ChainInvalidated);

        assert_eq!(lifecycle.source, SourceLaneState::Invalidated);
        assert_eq!(lifecycle.destination, DestinationLaneState::Invalidated);
        assert_eq!(lifecycle.chain, TransferChainState::Invalidated);
        assert_eq!(lifecycle.settlement, TransferSettlementState::Invalidated);
        assert_eq!(lifecycle.credit, TransferCreditState::Reversed);
    }

    #[derive(Debug, Clone)]
    enum Op {
        Notice,
        Ack,
        ObserveConfirming(u32),
        ObserveConfirmed(u32),
        Credit,
        Invalidate,
    }

    fn op_sequence() -> impl Strategy<Value = Vec<Op>> {
        prop::collection::vec(
            prop_oneof![
                Just(Op::Notice),
                Just(Op::Ack),
                (0u32..=32).prop_map(Op::ObserveConfirming),
                (1u32..=64).prop_map(Op::ObserveConfirmed),
                Just(Op::Credit),
                Just(Op::Invalidate),
            ],
            0..64,
        )
    }

    proptest! {
        #[test]
        fn prop_ack_is_state_preserving_after_terminal_confirmation(
            credited in any::<bool>(),
        ) {
            let mut lifecycle = TransferLifecycle::new();
            lifecycle.apply(LifecycleEvent::SourceSubmitted);
            lifecycle.apply(LifecycleEvent::ChainObserved {
                tx_hash: "abc",
                confirmations: 12,
                confirmed: true,
            });
            if credited {
                lifecycle.apply(LifecycleEvent::DestinationCredited);
            }

            let before = lifecycle.clone();
            lifecycle.apply(LifecycleEvent::AckReceived);

            prop_assert_eq!(lifecycle, before);
        }

        #[test]
        fn prop_lifecycle_invariants_hold_across_event_sequences(
            ops in op_sequence(),
        ) {
            let mut lifecycle = TransferLifecycle::new();
            lifecycle.apply(LifecycleEvent::SourceSubmitted);

            for op in ops {
                let before = lifecycle.clone();

                match op {
                    Op::Notice => lifecycle.apply(LifecycleEvent::DestinationNoticeReceived),
                    Op::Ack => lifecycle.apply(LifecycleEvent::AckReceived),
                    Op::ObserveConfirming(confirmations) => lifecycle.apply(LifecycleEvent::ChainObserved {
                        tx_hash: "abc",
                        confirmations,
                        confirmed: false,
                    }),
                    Op::ObserveConfirmed(confirmations) => lifecycle.apply(LifecycleEvent::ChainObserved {
                        tx_hash: "abc",
                        confirmations,
                        confirmed: true,
                    }),
                    Op::Credit => lifecycle.apply(LifecycleEvent::DestinationCredited),
                    Op::Invalidate => lifecycle.apply(LifecycleEvent::ChainInvalidated),
                }

                if matches!(op, Op::Ack | Op::Notice)
                    && before.source == SourceLaneState::Confirmed
                    && before.chain == TransferChainState::Confirmed
                {
                    prop_assert_eq!(lifecycle.clone(), before);
                }

                if lifecycle.source != SourceLaneState::Confirmed {
                    prop_assert_ne!(lifecycle.destination, DestinationLaneState::Credited);
                    prop_assert_ne!(
                        lifecycle.credit.clone(),
                        TransferCreditState::Credited
                    );
                }

                if lifecycle.credit == TransferCreditState::Credited {
                    prop_assert_eq!(lifecycle.source, SourceLaneState::Confirmed);
                    prop_assert_eq!(lifecycle.destination, DestinationLaneState::Credited);
                    prop_assert_eq!(lifecycle.chain.clone(), TransferChainState::Confirmed);
                    prop_assert_eq!(
                        lifecycle.settlement.clone(),
                        TransferSettlementState::Confirmed
                    );
                }

                if lifecycle.settlement == TransferSettlementState::Invalidated {
                    prop_assert_eq!(lifecycle.source, SourceLaneState::Invalidated);
                    prop_assert_eq!(lifecycle.destination, DestinationLaneState::Invalidated);
                    prop_assert_eq!(
                        lifecycle.chain.clone(),
                        TransferChainState::Invalidated
                    );
                    prop_assert!(
                        lifecycle.credit == TransferCreditState::None
                            || lifecycle.credit == TransferCreditState::Reversed
                    );
                }
            }
        }
    }
}
