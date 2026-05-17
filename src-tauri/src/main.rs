fn main() {
    if std::env::args().any(|argument| argument == "--run-due-database-backups") {
        if let Err(error) = axiomphp_lib::run_due_database_backups_once() {
            eprintln!("scheduled database backup sweep failed: {error}");
            std::process::exit(1);
        }

        return;
    }

    axiomphp_lib::run();
}
