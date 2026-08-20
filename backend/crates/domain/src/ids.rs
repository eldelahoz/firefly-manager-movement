//! Identificadores tipados del dominio.
//! Previene mezclar IDs de distintas entidades (p. ej. pasar un MovementId donde se espera un BatchId).

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(i64);

        impl $name {
            pub fn new(value: i64) -> Self {
                Self(value)
            }
            pub fn as_i64(self) -> i64 {
                self.0
            }
        }
        impl From<i64> for $name {
            fn from(value: i64) -> Self {
                Self(value)
            }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

define_id!(BatchId);
define_id!(MovementId);
define_id!(ReconciliationId);
define_id!(Ff3AccountId);
define_id!(Ff3TransactionId);
define_id!(MatchId);
define_id!(CreationId);
define_id!(CsvMappingId);
define_id!(DeletedAuditId);
