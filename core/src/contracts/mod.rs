pub mod fission;
pub mod fusion;

#[macro_export]
macro_rules! run_program {
    ($mod:tt, $stdin:expr) => {
        $crate::__dependencies::paste! {{
            use $crate::SerializeBytes;

            let program_id = $crate::Hash::from(mugraph_circuits::[< $mod:upper _ID >]);

            ::tracing::info!(
                id = %program_id,
                stdin = ?$stdin,
                "Running risc0 program...",
            );

            let mut prover = ::mugraph_circuits::Prover::new();
            let receipt = prover.prove(mugraph_circuits::[< $mod:upper _ELF >], &$stdin.to_bytes())?;

            let stdout = $crate::contracts::$mod::Stdout::from_slice(&prover.stdout)?;
            let journal = $crate::contracts::$mod::Output::from_slice(&receipt.journal.bytes)?;

            ::tracing::info!(
                id = %program_id,
                stdout = ?stdout,
                journal = ?journal,
                "Finished running risc0 program successfully..."
            );

            Ok::<_, $crate::Error>((stdout, journal))
        }};
    };
}
